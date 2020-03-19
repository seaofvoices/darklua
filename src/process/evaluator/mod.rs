mod lua_value;

pub use lua_value::*;

use crate::nodes::*;

/// A struct to convert an Expression node into a LuaValue object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluator {
    /// When evaluating expressions related to tables, this value tells the evaluator if
    /// metamethods can have side effects. For example, indexing a normal table in Lua does not
    /// have any side effects, but if the table is a metatable, it's __index metamethod can
    /// possibly have side effects (since it can be a function call).
    pure_metamethods: bool,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            pure_metamethods: false,
        }
    }
}

impl Evaluator {
    pub fn evaluate(&self, expression: &Expression) -> LuaValue {
        match expression {
            Expression::False => LuaValue::False,
            Expression::Function(_) => LuaValue::Function,
            Expression::Nil => LuaValue::Nil,
            Expression::Number(number) => LuaValue::from(number.compute_value()),
            Expression::String(string) => LuaValue::from(string.get_value()),
            Expression::Table(_) => LuaValue::Table,
            Expression::True => LuaValue::True,
            Expression::Binary(binary) => self.evaluate_binary(binary),
            Expression::Unary(unary) => self.evaluate_unary(unary),
            _ => LuaValue::Unknown
        }
    }

    pub fn has_side_effects(&self, expression: &Expression) -> bool {
        match expression {
            Expression::False
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::Nil
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::True
            | Expression::VariableArguments => false,
            | Expression::Binary(binary) => {
                if self.pure_metamethods {
                    self.has_side_effects(binary.left()) || self.has_side_effects(binary.left())
                } else {
                    let left = binary.left();
                    let right = binary.right();

                    self.maybe_table(&self.evaluate(left))
                    || self.maybe_table(&self.evaluate(right))
                    || self.has_side_effects(left)
                    || self.has_side_effects(right)
                }

            }
            | Expression::Unary(unary) => {
                if self.pure_metamethods {
                    self.has_side_effects(unary.get_expression())
                } else {
                    let sub_expression = unary.get_expression();

                    self.maybe_table(&self.evaluate(sub_expression))
                    || self.has_side_effects(sub_expression)
                }
            }
            Expression::Field(field) => self.field_has_side_effects(field),
            Expression::Index(index) => self.index_has_side_effects(index),
            Expression::Parenthese(sub_expression) => self.has_side_effects(sub_expression),
            Expression::Table(table) => table.get_entries().iter()
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
            TableEntry::Field(_, expression) => self.has_side_effects(expression),
            TableEntry::Index(key, value) => self.has_side_effects(key) || self.has_side_effects(value),
            TableEntry::Value(value) => self.has_side_effects(value),
        }
    }

    #[inline]
    fn field_has_side_effects(&self, field: &FieldExpression) -> bool {
        !self.pure_metamethods
        || self.prefix_has_side_effects(field.get_prefix())
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
            Prefix::Field(field) => self.field_has_side_effects(&field),
            Prefix::Identifier(_) => false,
            Prefix::Index(index) => self.index_has_side_effects(index),
            Prefix::Parenthese(sub_expression) => self.has_side_effects(sub_expression),
        }
    }

    #[inline]
    fn maybe_table(&self, value: &LuaValue) -> bool {
        match value {
            LuaValue::False
            | LuaValue::Function
            | LuaValue::Nil
            | LuaValue::Number(_)
            | LuaValue::String(_)
            | LuaValue::True => false,
            LuaValue::Table | LuaValue::Unknown => true
        }
    }

    fn evaluate_binary(&self, expression: &BinaryExpression) -> LuaValue {
        match expression.operator() {
            BinaryOperator::And => {
                self.evaluate(expression.left())
                    .map_if_truthy(|_| self.evaluate(expression.right()))
            }
            BinaryOperator::Or => {
                self.evaluate(expression.left())
                    .map_if_truthy_else(|left| left, || self.evaluate(expression.right()))
            }
            BinaryOperator::Equal => {
                self.evaluate_equal(
                    &self.evaluate(expression.left()),
                    &self.evaluate(expression.right())
                )
            }
            BinaryOperator::NotEqual => {
                let result = self.evaluate_equal(
                    &self.evaluate(expression.left()),
                    &self.evaluate(expression.right())
                );

                match result {
                    LuaValue::True => LuaValue::False,
                    LuaValue::False => LuaValue::True,
                    _ => LuaValue::Unknown,
                }
            }
            _ => LuaValue::Unknown,
        }
    }

    fn evaluate_equal(&self, left: &LuaValue, right: &LuaValue) -> LuaValue {
        match (left, right) {
            (LuaValue::Unknown, _) | (_, LuaValue::Unknown) => LuaValue::Unknown,
            (LuaValue::True, LuaValue::True)
            | (LuaValue::False, LuaValue::False)
            | (LuaValue::Nil, LuaValue::Nil) => LuaValue::True,
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::from(a == b),
            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::from(a == b),
            _ => LuaValue::False,
        }
    }

    fn evaluate_unary(&self, expression: &UnaryExpression) -> LuaValue {
        match expression.operator() {
            UnaryOperator::Not => {
                self.evaluate(expression.get_expression())
                    .is_truthy()
                    .map(|value| LuaValue::from(!value))
                    .unwrap_or(LuaValue::Unknown)
            }
            _ => LuaValue::Unknown,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! evaluate_expressions {
        ($($name:ident ($expression:expr) => $value:expr),*) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!($value, Evaluator::default().evaluate(&$expression.into()));
                }
            )*
        };
    }

    evaluate_expressions!(
        true_expression(Expression::True) => LuaValue::True,
        false_expression(Expression::False) => LuaValue::False,
        nil_expression(Expression::Nil) => LuaValue::Nil,
        number_expression(Expression::Number(DecimalNumber::new(0.0).into())) => LuaValue::Number(0.0),
        string_expression(StringExpression::from_value("foo")) => LuaValue::String("foo".to_owned()),
        table_expression(TableExpression::default()) => LuaValue::Table
    );

    mod binary_expressions {
        use super::*;

        macro_rules! evaluate_binary_expressions {
            ($($name:ident ($operator:expr, $left:expr, $right:expr) => $value:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        let binary = BinaryExpression::new($operator, $left.into(), $right.into());
                        assert_eq!($value, Evaluator::default().evaluate(&binary.into()));
                    }
                )*
            };
        }

        evaluate_binary_expressions!(
            true_and_number(
                BinaryOperator::And,
                Expression::True,
                Expression::Number(DecimalNumber::new(0.0).into())
            ) => LuaValue::Number(0.0),
            true_and_true(
                BinaryOperator::And,
                Expression::True,
                Expression::True
            ) => LuaValue::True,
            true_and_false(
                BinaryOperator::And,
                Expression::True,
                Expression::False
            ) => LuaValue::False,
            true_and_nil(
                BinaryOperator::And,
                Expression::True,
                Expression::Nil
            ) => LuaValue::Nil,
            true_and_string(
                BinaryOperator::And,
                Expression::True,
                Expression::String(StringExpression::from_value("foo"))
            ) => LuaValue::String("foo".to_owned()),
            true_and_table(
                BinaryOperator::And,
                Expression::True,
                TableExpression::default()
            ) => LuaValue::Table,
            nil_and_true(
                BinaryOperator::And,
                Expression::Nil,
                Expression::True
            ) => LuaValue::Nil,
            false_and_true(
                BinaryOperator::And,
                Expression::False,
                Expression::True
            ) => LuaValue::False,
            true_or_number(
                BinaryOperator::Or,
                Expression::True,
                Expression::Number(DecimalNumber::new(0.0).into())
            ) => LuaValue::True,
            true_or_true(
                BinaryOperator::Or,
                Expression::True,
                Expression::True
            ) => LuaValue::True,
            true_or_false(
                BinaryOperator::Or,
                Expression::True,
                Expression::False
            ) => LuaValue::True,
            true_or_nil(
                BinaryOperator::Or,
                Expression::True,
                Expression::Nil
            ) => LuaValue::True,
            true_or_string(
                BinaryOperator::Or,
                Expression::True,
                Expression::String(StringExpression::from_value("foo"))
            ) => LuaValue::True,
            nil_or_true(
                BinaryOperator::Or,
                Expression::Nil,
                Expression::True
            ) => LuaValue::True,
            nil_or_false(
                BinaryOperator::Or,
                Expression::Nil,
                Expression::False
            ) => LuaValue::False,
            nil_or_nil(
                BinaryOperator::Or,
                Expression::Nil,
                Expression::Nil
            ) => LuaValue::Nil
        );

        macro_rules! evaluate_equality {
            ($($name:ident ($left:expr, $right:expr) => $value:expr),*) => {
                $(
                    mod $name {
                        use super::*;

                        #[test]
                        fn equal() {
                            let binary = BinaryExpression::new(
                                BinaryOperator::Equal,
                                $left.into(),
                                $right.into(),
                            );

                            assert_eq!($value, Evaluator::default().evaluate(&binary.into()));

                            let binary = BinaryExpression::new(
                                BinaryOperator::Equal,
                                $right.into(),
                                $left.into(),
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
                                $left.into(),
                                $right.into()
                            );

                            assert_eq!(value, Evaluator::default().evaluate(&binary.into()));

                            let binary = BinaryExpression::new(
                                BinaryOperator::NotEqual,
                                $right.into(),
                                $left.into(),
                            );

                            assert_eq!(value, Evaluator::default().evaluate(&binary.into()));
                        }
                    }
                )*
            };
        }

        evaluate_equality!(
            true_true(Expression::True, Expression::True) => LuaValue::True,
            false_false(Expression::False, Expression::False) => LuaValue::True,
            nil_nil(Expression::Nil, Expression::Nil) => LuaValue::True,
            same_strings(
                StringExpression::from_value("foo"),
                StringExpression::from_value("foo")
            ) => LuaValue::True,
            same_numbers(
                Expression::Number(DecimalNumber::new(0.0).into()),
                Expression::Number(DecimalNumber::new(0.0).into())
            ) => LuaValue::True,
            true_false(Expression::True, Expression::False) => LuaValue::False,
            true_nil(Expression::True, Expression::False) => LuaValue::False,
            different_numbers(
                Expression::Number(DecimalNumber::new(1.0).into()),
                Expression::Number(DecimalNumber::new(10.0).into())
            ) => LuaValue::False,
            different_strings(
                StringExpression::from_value("foo"),
                StringExpression::from_value("bar")
            ) => LuaValue::False
        );
    }

    mod unary_expressions {
        use super::*;

        macro_rules! evaluate_unary_expressions {
            ($($name:ident ($operator:expr, $input:expr) => $value:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        let unary = UnaryExpression::new($operator, $input.into());
                        assert_eq!($value, Evaluator::default().evaluate(&unary.into()));
                    }
                )*
            };
        }

        evaluate_unary_expressions!(
            not_true(UnaryOperator::Not, Expression::True) => LuaValue::False,
            not_false(UnaryOperator::Not, Expression::False) => LuaValue::True,
            not_nil(UnaryOperator::Not, Expression::Nil) => LuaValue::True,
            not_table(UnaryOperator::Not, TableExpression::default()) => LuaValue::False,
            not_string(UnaryOperator::Not, StringExpression::from_value("foo")) => LuaValue::False,
            not_number(
                UnaryOperator::Not,
                Expression::Number(DecimalNumber::new(10.0).into())
            ) => LuaValue::False,
            not_identifier(UnaryOperator::Not, Expression::Identifier("foo".to_owned())) => LuaValue::Unknown
        );
    }

    macro_rules! has_side_effects {
        ($($name:ident ($expression:expr)),*) => {
            $(
                #[test]
                fn $name() {
                    assert!(Evaluator::default().has_side_effects(&$expression.into()));
                }
            )*
        };
    }

    macro_rules! has_no_side_effects {
        ($($name:ident ($expression:expr)),*) => {
            $(
                #[test]
                fn $name() {
                    assert!(!Evaluator::default().has_side_effects(&$expression.into()));
                }
            )*
        };
    }

    has_side_effects!(
        call_to_unknown_function(FunctionCall::from_name("foo"))
    );

    has_no_side_effects!(
        true_value(Expression::True),
        false_value(Expression::False),
        nil_value(Expression::Nil),
        number_value(Expression::Number(DecimalNumber::new(0.0).into())),
        string_value(StringExpression::from_value("")),
        identifier(Expression::Identifier("foo".to_owned())),
        identifier_in_parentheses(Expression::Parenthese(
            Box::new(Expression::Identifier("foo".to_owned()))
        ))
    );
}
