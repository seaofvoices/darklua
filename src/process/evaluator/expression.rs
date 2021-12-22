use crate::nodes::*;

use super::{FunctionValue, LuaValue, TableValue, VirtualLuaExecution};

/// A struct to convert an Expression node into a LuaValue object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evaluator<'a> {
    pure_metamethods: bool,
    state: Option<&'a VirtualLuaExecution>,
}

impl<'a> Evaluator<'a> {
    pub fn new(state: &'a VirtualLuaExecution) -> Self {
        Self {
            pure_metamethods: false,
            state: Some(state),
        }
    }

    /// When evaluating expressions related to tables, this value tells the evaluator if
    /// metamethods can have side effects. For example, indexing a normal table in Lua does not
    /// have any side effects, but if the table is a metatable, it's `__index` metamethod can
    /// possibly have side effects (since it can be a function call).
    pub fn assume_pure_metamethods(mut self) -> Self {
        self.pure_metamethods = true;
        self
    }

    pub fn evaluate(&self, expression: &Expression) -> LuaValue {
        match expression {
            Expression::False(_) => LuaValue::False,
            Expression::Function(_) => FunctionValue::Unknown.into(),
            Expression::Nil(_) => LuaValue::Nil,
            Expression::Number(number) => LuaValue::from(number.compute_value()),
            Expression::String(string) => LuaValue::from(string.get_value()),
            Expression::Table(table) => self.evaluate_table(table),
            Expression::True(_) => LuaValue::True,
            Expression::Binary(binary) => self.evaluate_binary(binary),
            Expression::Unary(unary) => self.evaluate_unary(unary),
            Expression::Parenthese(parenthese) => self.evaluate_parenthese(parenthese),
            Expression::Identifier(identifier) => self.evaluate_identifier(identifier),
            Expression::Field(field) => self.evaluate_field(field),
            Expression::Index(index) => self.evaluate_index(index),
            Expression::Call(call) => self.evaluate_call(call),
            Expression::VariableArguments(_) => LuaValue::Unknown,
        }
    }

    pub fn has_side_effects(&self, expression: &Expression) -> bool {
        match expression {
            Expression::False(_)
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::True(_)
            | Expression::VariableArguments(_) => false,
            Expression::Binary(binary) => {
                let left = binary.left();
                let right = binary.right();

                let left_value = self.evaluate(left);
                let left_side_effect = self.has_side_effects(binary.left());

                match binary.operator() {
                    BinaryOperator::And => {
                        if left_value.is_truthy().unwrap_or(true) {
                            left_side_effect || self.has_side_effects(binary.right())
                        } else {
                            left_side_effect
                        }
                    }
                    BinaryOperator::Or => {
                        if left_value.is_truthy().unwrap_or(false) {
                            left_side_effect
                        } else {
                            left_side_effect || self.has_side_effects(binary.right())
                        }
                    }
                    _ => {
                        if self.pure_metamethods {
                            left_side_effect || self.has_side_effects(binary.right())
                        } else {
                            self.maybe_metatable(&left_value)
                                || self.maybe_metatable(&self.evaluate(right))
                                || self.has_side_effects(left)
                                || self.has_side_effects(right)
                        }
                    }
                }
            }
            Expression::Unary(unary) => {
                if self.pure_metamethods || matches!(unary.operator(), UnaryOperator::Not) {
                    self.has_side_effects(unary.get_expression())
                } else {
                    let sub_expression = unary.get_expression();

                    self.maybe_metatable(&self.evaluate(sub_expression))
                        || self.has_side_effects(sub_expression)
                }
            }
            Expression::Field(field) => self.field_has_side_effects(field),
            Expression::Index(index) => self.index_has_side_effects(index),
            Expression::Parenthese(parenthese) => {
                self.has_side_effects(parenthese.inner_expression())
            }
            Expression::Table(table) => table
                .get_entries()
                .iter()
                .any(|entry| self.table_entry_has_side_effects(entry)),
            Expression::Call(call) => self.call_has_side_effects(call),
        }
    }

    #[inline]
    fn call_has_side_effects(&self, _call: &FunctionCall) -> bool {
        true
    }

    #[inline]
    fn table_entry_has_side_effects(&self, entry: &TableEntry) -> bool {
        match entry {
            TableEntry::Field(entry) => self.has_side_effects(entry.get_value()),
            TableEntry::Index(entry) => {
                self.has_side_effects(entry.get_key()) || self.has_side_effects(entry.get_value())
            }
            TableEntry::Value(value) => self.has_side_effects(value),
        }
    }

    #[inline]
    fn field_has_side_effects(&self, field: &FieldExpression) -> bool {
        !self.pure_metamethods || self.prefix_has_side_effects(field.get_prefix())
    }

    #[inline]
    fn index_has_side_effects(&self, index: &IndexExpression) -> bool {
        !self.pure_metamethods
            || self.has_side_effects(index.get_index())
            || self.prefix_has_side_effects(index.get_prefix())
    }

    fn prefix_has_side_effects(&self, prefix: &Prefix) -> bool {
        match prefix {
            Prefix::Call(call) => self.call_has_side_effects(call),
            Prefix::Field(field) => self.field_has_side_effects(field),
            Prefix::Identifier(_) => false,
            Prefix::Index(index) => self.index_has_side_effects(index),
            Prefix::Parenthese(sub_expression) => {
                self.has_side_effects(sub_expression.inner_expression())
            }
        }
    }

    #[inline]
    fn maybe_metatable(&self, value: &LuaValue) -> bool {
        match value {
            LuaValue::False
            | LuaValue::Function(_)
            | LuaValue::Nil
            | LuaValue::Number(_)
            | LuaValue::String(_)
            | LuaValue::Table(_) // TODO
            | LuaValue::True => false,
            LuaValue::TableRef(_) // TODO
            | LuaValue::Unknown => true,
            LuaValue::Tuple(tuple) => self.maybe_metatable(tuple.as_single_value()),
        }
    }

    fn evaluate_binary(&self, expression: &BinaryExpression) -> LuaValue {
        match expression.operator() {
            BinaryOperator::And => self
                .evaluate(expression.left())
                .map_if_truthy(|_| self.evaluate(expression.right())),
            BinaryOperator::Or => self
                .evaluate(expression.left())
                .map_if_truthy_else(|left| left, || self.evaluate(expression.right())),
            BinaryOperator::Equal => self.evaluate_equal(
                &self.evaluate(expression.left()),
                &self.evaluate(expression.right()),
            ),
            BinaryOperator::NotEqual => {
                let result = self.evaluate_equal(
                    &self.evaluate(expression.left()),
                    &self.evaluate(expression.right()),
                );

                match result {
                    LuaValue::True => LuaValue::False,
                    LuaValue::False => LuaValue::True,
                    _ => LuaValue::Unknown,
                }
            }
            BinaryOperator::Plus => self.evaluate_math(expression, |a, b| a + b),
            BinaryOperator::Minus => self.evaluate_math(expression, |a, b| a - b),
            BinaryOperator::Asterisk => self.evaluate_math(expression, |a, b| a * b),
            BinaryOperator::Slash => self.evaluate_math(expression, |a, b| a / b),
            BinaryOperator::Caret => self.evaluate_math(expression, |a, b| a.powf(b)),
            BinaryOperator::Percent => {
                self.evaluate_math(expression, |a, b| a - b * (a / b).floor())
            }
            BinaryOperator::Concat => {
                match (
                    self.evaluate(expression.left()).string_coercion(),
                    self.evaluate(expression.right()).string_coercion(),
                ) {
                    (LuaValue::String(mut left), LuaValue::String(right)) => {
                        left.push_str(&right);
                        LuaValue::String(left)
                    }
                    _ => LuaValue::Unknown,
                }
            }
            BinaryOperator::LowerThan => self.evaluate_relational(expression, |a, b| a < b),
            BinaryOperator::LowerOrEqualThan => self.evaluate_relational(expression, |a, b| a <= b),
            BinaryOperator::GreaterThan => self.evaluate_relational(expression, |a, b| a > b),
            BinaryOperator::GreaterOrEqualThan => {
                self.evaluate_relational(expression, |a, b| a >= b)
            }
        }
    }

    fn evaluate_equal(&self, left: &LuaValue, right: &LuaValue) -> LuaValue {
        match (left, right) {
            (LuaValue::Unknown, _) | (_, LuaValue::Unknown) => LuaValue::Unknown,
            (LuaValue::True, LuaValue::True)
            | (LuaValue::False, LuaValue::False)
            | (LuaValue::Nil, LuaValue::Nil) => LuaValue::True,
            (LuaValue::Number(a), LuaValue::Number(b)) => {
                LuaValue::from((a - b).abs() < f64::EPSILON)
            }
            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::from(a == b),
            _ => LuaValue::False,
        }
    }

    fn evaluate_math<F>(&self, expression: &BinaryExpression, operation: F) -> LuaValue
    where
        F: Fn(f64, f64) -> f64,
    {
        let left = self.evaluate(expression.left()).number_coercion();

        if let LuaValue::Number(left) = left {
            let right = self.evaluate(expression.right()).number_coercion();

            if let LuaValue::Number(right) = right {
                LuaValue::Number(operation(left, right))
            } else {
                LuaValue::Unknown
            }
        } else {
            LuaValue::Unknown
        }
    }

    fn evaluate_relational<F>(&self, expression: &BinaryExpression, operation: F) -> LuaValue
    where
        F: Fn(f64, f64) -> bool,
    {
        let left = self.evaluate(expression.left());

        match left {
            LuaValue::Number(left) => {
                let right = self.evaluate(expression.right());

                if let LuaValue::Number(right) = right {
                    if operation(left, right) {
                        LuaValue::True
                    } else {
                        LuaValue::False
                    }
                } else {
                    LuaValue::Unknown
                }
            }
            LuaValue::String(left) => {
                let right = self.evaluate(expression.right());

                if let LuaValue::String(right) = right {
                    self.compare_strings(&left, &right, expression.operator())
                } else {
                    LuaValue::Unknown
                }
            }
            _ => LuaValue::Unknown,
        }
    }

    fn compare_strings(&self, left: &str, right: &str, operator: BinaryOperator) -> LuaValue {
        LuaValue::from(match operator {
            BinaryOperator::Equal => left == right,
            BinaryOperator::NotEqual => left != right,
            BinaryOperator::LowerThan => left < right,
            BinaryOperator::LowerOrEqualThan => left <= right,
            BinaryOperator::GreaterThan => left > right,
            BinaryOperator::GreaterOrEqualThan => left >= right,
            _ => return LuaValue::Unknown,
        })
    }

    fn evaluate_unary(&self, expression: &UnaryExpression) -> LuaValue {
        match expression.operator() {
            UnaryOperator::Not => self
                .evaluate(expression.get_expression())
                .is_truthy()
                .map(|value| LuaValue::from(!value))
                .unwrap_or(LuaValue::Unknown),
            UnaryOperator::Minus => {
                match self.evaluate(expression.get_expression()).number_coercion() {
                    LuaValue::Number(value) => LuaValue::from(-value),
                    _ => LuaValue::Unknown,
                }
            }
            _ => LuaValue::Unknown,
        }
    }

    fn evaluate_parenthese(&self, parenthese: &ParentheseExpression) -> LuaValue {
        self.evaluate(parenthese.inner_expression())
            .coerce_to_single_value()
    }

    fn evaluate_identifier(&self, _identifier: &Identifier) -> LuaValue {
        LuaValue::Unknown
    }

    fn evaluate_field(&self, field: &FieldExpression) -> LuaValue {
        match self.evaluate_prefix(field.get_prefix()).coerce_to_single_value() {
            LuaValue::Table(_table) => {
                // TODO: table.get(&field.get_field().get_name().to_owned().into())
                //     .clone()
                LuaValue::Unknown
            }
            LuaValue::TableRef(_id) => {
                LuaValue::Unknown
            }
            LuaValue::Nil
            | LuaValue::Function(_)
            | LuaValue::Number(_)
            | LuaValue::String(_) // TODO: strings can be indexed
            | LuaValue::True
            | LuaValue::False
            | LuaValue::Unknown => LuaValue::Unknown,
            // unreachable because of the call to `coerce_to_single_value`
            LuaValue::Tuple(_) => LuaValue::Unknown,
        }
    }

    fn evaluate_index(&self, _index: &IndexExpression) -> LuaValue {
        // TODO
        LuaValue::Unknown
    }

    fn evaluate_call(&self, _call: &FunctionCall) -> LuaValue {
        LuaValue::Unknown
    }

    fn evaluate_prefix(&self, prefix: &Prefix) -> LuaValue {
        match prefix {
            Prefix::Field(field) => self.evaluate_field(field),
            Prefix::Identifier(identifier) => self.evaluate_identifier(identifier),
            Prefix::Index(index) => self.evaluate_index(index),
            Prefix::Parenthese(parenthese) => self.evaluate_parenthese(parenthese),
            Prefix::Call(call) => self.evaluate_call(call),
        }
    }

    fn evaluate_table(&self, _table: &TableExpression) -> LuaValue {
        // TODO
        TableValue::default().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! evaluate_expressions {
        ($($name:ident ($expression:expr) => $value:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!($value, Evaluator::default().evaluate(&$expression.into()));
                }
            )*
        };
    }

    evaluate_expressions!(
        true_expression(Expression::from(true)) => LuaValue::True,
        false_expression(Expression::from(false)) => LuaValue::False,
        nil_expression(Expression::nil()) => LuaValue::Nil,
        number_expression(DecimalNumber::new(0.0)) => LuaValue::Number(0.0),
        string_expression(StringExpression::from_value("foo")) => LuaValue::String("foo".to_owned()),
        true_wrapped_in_parens(ParentheseExpression::new(true)) => LuaValue::True,
        false_wrapped_in_parens(ParentheseExpression::new(false)) => LuaValue::False,
        nil_wrapped_in_parens(ParentheseExpression::new(Expression::nil())) => LuaValue::Nil,
        number_wrapped_in_parens(ParentheseExpression::new(DecimalNumber::new(0.0)))
            => LuaValue::Number(0.0),
        string_wrapped_in_parens(ParentheseExpression::new(StringExpression::from_value("foo")))
            => LuaValue::String("foo".to_owned()),
        table_expression(TableExpression::default()) => LuaValue::empty_table(),
    );

    mod binary_expressions {
        use super::*;

        macro_rules! evaluate_binary_expressions {
            ($($name:ident ($operator:expr, $left:expr, $right:expr) => $expect:expr),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let binary = BinaryExpression::new($operator, $left, $right);

                        let result = Evaluator::default().evaluate(&binary.into());
                        let expect = $expect.into();

                        match (&expect, &result) {
                            (LuaValue::Number(expect_float), LuaValue::Number(result))=> {
                                if expect_float.is_nan() {
                                    assert!(result.is_nan(), "{} should be NaN", result);
                                } else if expect_float.is_infinite() {
                                    assert!(result.is_infinite(), "{} should be infinite", result);
                                    assert_eq!(expect_float.is_sign_positive(), result.is_sign_positive());
                                } else {
                                    assert!(
                                        (expect_float - result).abs() < f64::EPSILON,
                                        "{} does not approximate {}", result, expect_float
                                    );
                                }
                            }
                            _ => {
                                assert_eq!(expect, result.into());
                            }
                        }
                    }
                )*
            };
        }

        evaluate_binary_expressions!(
            true_and_number(
                BinaryOperator::And,
                true,
                Expression::Number(DecimalNumber::new(0.0).into())
            ) => LuaValue::Number(0.0),
            true_and_true(
                BinaryOperator::And,
                true,
                true
            ) => LuaValue::True,
            true_and_false(
                BinaryOperator::And,
                true,
                false
            ) => LuaValue::False,
            true_and_nil(
                BinaryOperator::And,
                true,
                Expression::nil()
            ) => LuaValue::Nil,
            true_and_string(
                BinaryOperator::And,
                true,
                Expression::String(StringExpression::from_value("foo"))
            ) => LuaValue::String("foo".to_owned()),
            true_and_table(
                BinaryOperator::And,
                true,
                TableExpression::default()
            ) => TableValue::default(),
            nil_and_true(
                BinaryOperator::And,
                Expression::nil(),
                true
            ) => LuaValue::Nil,
            false_and_true(
                BinaryOperator::And,
                false,
                true
            ) => LuaValue::False,
            true_or_number(
                BinaryOperator::Or,
                true,
                Expression::Number(DecimalNumber::new(0.0).into())
            ) => LuaValue::True,
            true_or_true(
                BinaryOperator::Or,
                true,
                true
            ) => LuaValue::True,
            true_or_false(
                BinaryOperator::Or,
                true,
                false
            ) => LuaValue::True,
            true_or_nil(
                BinaryOperator::Or,
                true,
                Expression::nil()
            ) => LuaValue::True,
            true_or_string(
                BinaryOperator::Or,
                true,
                Expression::String(StringExpression::from_value("foo"))
            ) => LuaValue::True,
            nil_or_true(
                BinaryOperator::Or,
                Expression::nil(),
                true
            ) => LuaValue::True,
            nil_or_false(
                BinaryOperator::Or,
                Expression::nil(),
                false
            ) => LuaValue::False,
            nil_or_nil(
                BinaryOperator::Or,
                Expression::nil(),
                Expression::nil()
            ) => LuaValue::Nil,
            one_plus_two(
                BinaryOperator::Plus,
                Expression::from(1.0),
                Expression::from(2.0)
            ) => LuaValue::Number(3.0),
            one_minus_two(
                BinaryOperator::Minus,
                Expression::from(1.0),
                Expression::from(2.0)
            ) => LuaValue::Number(-1.0),
            three_times_four(
                BinaryOperator::Asterisk,
                Expression::from(3.0),
                Expression::from(4.0)
            ) => LuaValue::Number(12.0),
            twelve_divided_by_four(
                BinaryOperator::Slash,
                Expression::from(12.0),
                Expression::from(4.0)
            ) => LuaValue::Number(3.0),
            one_divided_by_zero(
                BinaryOperator::Slash,
                Expression::from(1.0),
                Expression::from(0.0)
            ) => LuaValue::Number(std::f64::INFINITY),
            zero_divided_by_zero(
                BinaryOperator::Slash,
                Expression::from(0.0),
                Expression::from(0.0)
            ) => LuaValue::Number(std::f64::NAN),
            five_mod_two(
                BinaryOperator::Percent,
                Expression::from(5.0),
                Expression::from(2.0)
            ) => LuaValue::Number(1.0),
            minus_five_mod_two(
                BinaryOperator::Percent,
                Expression::from(-5.0),
                Expression::from(2.0)
            ) => LuaValue::Number(1.0),
            minus_five_mod_minus_two(
                BinaryOperator::Percent,
                Expression::from(-5.0),
                Expression::from(-2.0)
            ) => LuaValue::Number(-1.0),
            five_point_two_mod_two(
                BinaryOperator::Percent,
                Expression::from(5.5),
                Expression::from(2.0)
            ) => LuaValue::Number(1.5),
            five_pow_two(
                BinaryOperator::Caret,
                Expression::from(5.0),
                Expression::from(2.0)
            ) => LuaValue::Number(25.0),
            string_number_plus_string_number(
                BinaryOperator::Plus,
                StringExpression::from_value("2"),
                StringExpression::from_value("3")
            ) => LuaValue::Number(5.0),
            concat_strings(
                BinaryOperator::Concat,
                StringExpression::from_value("2"),
                StringExpression::from_value("3")
            ) => LuaValue::from("23"),
            concat_string_with_number(
                BinaryOperator::Concat,
                StringExpression::from_value("foo"),
                11.0
            ) => LuaValue::from("foo11"),
            concat_number_with_string(
                BinaryOperator::Concat,
                11.0,
                StringExpression::from_value("foo")
            ) => LuaValue::from("11foo"),
            concat_number_with_number(
                BinaryOperator::Concat,
                11.0,
                33.0
            ) => LuaValue::from("1133"),
            concat_number_with_negative_number(
                BinaryOperator::Concat,
                11.0,
                -33.0
            ) => LuaValue::from("11-33"),
            concat_empty_strings(
                BinaryOperator::Concat,
                StringExpression::empty(),
                StringExpression::empty()
            ) => LuaValue::from(""),
            number_lower_than_string(
                BinaryOperator::LowerThan,
                1.0,
                StringExpression::empty()
            ) => LuaValue::Unknown,
            number_string_greater_than_number(
                BinaryOperator::GreaterThan,
                StringExpression::from_value("100"),
                1.0
            ) => LuaValue::Unknown,
            number_string_greater_or_equal_than_number(
                BinaryOperator::GreaterOrEqualThan,
                StringExpression::from_value("100"),
                100.0
            ) => LuaValue::Unknown,
            number_lower_or_equal_than_number_string(
                BinaryOperator::GreaterOrEqualThan,
                100.0,
                StringExpression::from_value("100")
            ) => LuaValue::Unknown,
        );

        macro_rules! evaluate_equality {
            ($($name:ident ($left:expr, $right:expr) => $value:expr),* $(,)?) => {
                $(
                    mod $name {
                        use super::*;

                        #[test]
                        fn equal() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::Equal,
                                $left,
                                $right,
                            );

                            assert_eq!($value, Evaluator::default().evaluate(&binary.into()));

                            let binary = BinaryExpression::new(
                                BinaryOperator::Equal,
                                $right,
                                $left,
                            );

                            assert_eq!($value, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn not_equal() {
                            let value = match $value {
                                LuaValue::True => LuaValue::False,
                                LuaValue::False => LuaValue::True,
                                _ => LuaValue::Unknown
                            };
                            let binary = BinaryExpression::new(
                                BinaryOperator::NotEqual,
                                $left,
                                $right,
                            );

                            assert_eq!(value, Evaluator::default().evaluate(&binary.into()));

                            let binary = BinaryExpression::new(
                                BinaryOperator::NotEqual,
                                $right,
                                $left,
                            );

                            assert_eq!(value, Evaluator::default().evaluate(&binary.into()));
                        }
                    }
                )*
            };
        }

        evaluate_equality!(
            true_true(Expression::from(true), Expression::from(true)) => LuaValue::True,
            false_false(Expression::from(false), Expression::from(false)) => LuaValue::True,
            nil_nil(Expression::nil(), Expression::nil()) => LuaValue::True,
            same_strings(
                StringExpression::from_value("foo"),
                StringExpression::from_value("foo")
            ) => LuaValue::True,
            same_numbers(
                Expression::Number(DecimalNumber::new(0.0).into()),
                Expression::Number(DecimalNumber::new(0.0).into())
            ) => LuaValue::True,
            true_false(Expression::from(true), Expression::from(false)) => LuaValue::False,
            true_nil(Expression::from(true), Expression::from(false)) => LuaValue::False,
            different_numbers(
                Expression::Number(DecimalNumber::new(1.0).into()),
                Expression::Number(DecimalNumber::new(10.0).into())
            ) => LuaValue::False,
            different_strings(
                StringExpression::from_value("foo"),
                StringExpression::from_value("bar")
            ) => LuaValue::False,
        );

        macro_rules! evaluate_equality_with_relational_operators {
            ($($name:ident => $value:expr),* $(,)?) => {
                $(
                    mod $name {
                        use super::*;

                        #[test]
                        fn lower() {
                            let value: Expression = $value.into();
                            let binary = BinaryExpression::new(BinaryOperator::LowerThan, value.clone(), value);
                            assert_eq!(LuaValue::False, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn lower_or_equal() {
                            let value: Expression = $value.into();
                            let binary = BinaryExpression::new(BinaryOperator::LowerOrEqualThan, value.clone(), value);
                            assert_eq!(LuaValue::True, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn greater() {
                            let value: Expression = $value.into();
                            let binary = BinaryExpression::new(BinaryOperator::GreaterThan, value.clone(), value);
                            assert_eq!(LuaValue::False, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn greater_or_equal() {
                            let value: Expression = $value.into();
                            let binary = BinaryExpression::new(BinaryOperator::GreaterOrEqualThan, value.clone(), value);
                            assert_eq!(LuaValue::True, Evaluator::default().evaluate(&binary.into()));
                        }
                    }
                )*
            };
        }

        evaluate_equality_with_relational_operators!(
            zero => 1.0,
            one => 1.0,
            hundred => 100.0,
            string => StringExpression::from_value("var"),
        );

        macro_rules! evaluate_strict_relational_operators {
            ($($name_lower:ident($lower:expr) < $name_greater:ident($greater:expr)),* $(,)?) => {
                mod lower_or_greater_than {
                    use super::*;
                    paste::paste! {

                    $(
                        #[test]
                        fn [<$name_lower _lower_than_ $name_greater>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::LowerThan,
                                $lower,
                                $greater,
                            );
                            assert_eq!(LuaValue::True, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_lower _lower_or_equal_than_ $name_greater>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::LowerOrEqualThan,
                                $lower,
                                $greater,
                            );
                            assert_eq!(LuaValue::True, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_lower _greater_than_ $name_greater>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::GreaterThan,
                                $lower,
                                $greater,
                            );
                            assert_eq!(LuaValue::False, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_lower _greater_or_equal_than_ $name_greater>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::GreaterOrEqualThan,
                                $lower,
                                $greater,
                            );
                            assert_eq!(LuaValue::False, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_greater _lower_than_ $name_lower>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::LowerThan,
                                $greater,
                                $lower,
                            );
                            assert_eq!(LuaValue::False, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_greater _lower_or_equal_than_ $name_lower>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::LowerOrEqualThan,
                                $greater,
                                $lower,
                            );
                            assert_eq!(LuaValue::False, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_greater _greater_than_ $name_lower>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::GreaterThan,
                                $greater,
                                $lower,
                            );
                            assert_eq!(LuaValue::True, Evaluator::default().evaluate(&binary.into()));
                        }

                        #[test]
                        fn [<$name_greater _greater_or_equal_than_ $name_lower>]() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::GreaterOrEqualThan,
                                $greater,
                                $lower,
                            );
                            assert_eq!(LuaValue::True, Evaluator::default().evaluate(&binary.into()));
                        }
                    )*

                    }
                }
            };
        }

        evaluate_strict_relational_operators!(
            one(1.0) < hundred(100.0),
            minus_15(-15.0) < minus_2_5(-2.5),
            string_a(StringExpression::from_value("a"))
                < string_b(StringExpression::from_value("b")),
            string_a(StringExpression::from_value("a"))
                < string_aa(StringExpression::from_value("aa")),
            string_1(StringExpression::from_value("1"))
                < string_a(StringExpression::from_value("a")),
            string_111(StringExpression::from_value("111"))
                < string_a(StringExpression::from_value("a")),
            empty_string(StringExpression::from_value(""))
                < string_colon(StringExpression::from_value(":")),
        );
    }

    mod unary_expressions {
        use super::*;
        use UnaryOperator::*;

        macro_rules! evaluate_unary_expressions {
            ($($name:ident ($operator:expr, $input:expr) => $value:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        let unary = UnaryExpression::new($operator, $input);
                        assert_eq!($value, Evaluator::default().evaluate(&unary.into()));
                    }
                )*
            };
        }

        evaluate_unary_expressions!(
            not_true(Not, Expression::from(true)) => LuaValue::False,
            not_false(Not, Expression::from(false)) => LuaValue::True,
            not_nil(Not, Expression::nil()) => LuaValue::True,
            not_table(Not, TableExpression::default()) => LuaValue::False,
            not_string(Not, StringExpression::from_value("foo")) => LuaValue::False,
            not_number(
                Not,
                Expression::Number(DecimalNumber::new(10.0).into())
            ) => LuaValue::False,
            not_identifier(Not, Expression::identifier("foo")) => LuaValue::Unknown,
            minus_one(Minus, DecimalNumber::new(1.0)) => LuaValue::from(-1.0),
            minus_negative_number(Minus, DecimalNumber::new(-5.0)) => LuaValue::from(5.0),
            minus_string_converted_to_number(Minus, StringExpression::from_value("1")) => LuaValue::from(-1.0)
        );
    }

    macro_rules! has_side_effects {
        ($($name:ident => $expression:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert!(Evaluator::default().has_side_effects(&$expression.into()));
                }
            )*
        };
    }

    macro_rules! has_no_side_effects {
        ($($name:ident => $expression:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert!(!Evaluator::default().has_side_effects(&$expression.into()));
                }
            )*
        };
    }

    has_side_effects!(
        call_to_unknown_function => FunctionCall::from_name("foo"),
        binary_true_and_call => BinaryExpression::new(
            BinaryOperator::And,
            Expression::from(true),
            FunctionCall::from_name("foo"),
        ),
        binary_false_or_call => BinaryExpression::new(
            BinaryOperator::Or,
            Expression::from(false),
            FunctionCall::from_name("var"),
        ),
        addition_unknown_variable_and_number => BinaryExpression::new(
            BinaryOperator::Plus,
            Expression::identifier("var"),
            1.0,
        ),
        addition_number_with_unknown_variable => BinaryExpression::new(
            BinaryOperator::Plus,
            1.0,
            Expression::identifier("var"),
        ),
        unary_minus_on_variable => UnaryExpression::new(UnaryOperator::Minus, Identifier::new("var")),
        length_on_variable => UnaryExpression::new(UnaryOperator::Length, Identifier::new("var")),
        field_index => FieldExpression::new(Identifier::new("var"), "field"),
        table_value_with_call_in_entry => TableExpression::default()
            .append_array_value(FunctionCall::from_name("call")),
    );

    has_no_side_effects!(
        true_value => Expression::from(true),
        false_value => Expression::from(false),
        nil_value => Expression::nil(),
        table_value => TableExpression::default(),
        number_value => Expression::Number(DecimalNumber::new(0.0).into()),
        string_value => StringExpression::from_value(""),
        identifier => Expression::identifier("foo"),
        identifier_in_parentheses => Expression::identifier("foo").in_parentheses(),
        binary_false_and_call => BinaryExpression::new(
            BinaryOperator::And,
            Expression::from(false),
            FunctionCall::from_name("foo"),
        ),
        binary_true_or_call => BinaryExpression::new(
            BinaryOperator::Or,
            Expression::from(true),
            FunctionCall::from_name("foo"),
        ),
        not_variable => UnaryExpression::new(UnaryOperator::Not, Identifier::new("var")),
    );

    mod assume_pure_metamethods {
        use super::*;

        macro_rules! has_no_side_effects {
            ($($name:ident => $expression:expr),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let evaluator = Evaluator::default().assume_pure_metamethods();
                        assert!(!evaluator.has_side_effects(&$expression.into()));
                    }
                )*
            };
        }

        has_no_side_effects!(
            addition_unknown_variable_and_number => BinaryExpression::new(
                BinaryOperator::Plus,
                Expression::identifier("foo"),
                1.0,
            ),
            addition_number_with_unknown_variable => BinaryExpression::new(
                BinaryOperator::Plus,
                1.0,
                Expression::identifier("foo"),
            ),
            unary_minus_on_variable => UnaryExpression::new(UnaryOperator::Minus, Identifier::new("var")),
            length_on_variable => UnaryExpression::new(UnaryOperator::Length, Identifier::new("var")),
            not_on_variable => UnaryExpression::new(UnaryOperator::Not, Identifier::new("var")),
            field_index => FieldExpression::new(Identifier::new("var"), "field"),
        );
    }
}
