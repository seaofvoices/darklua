mod evaluator_storage;
mod function_value;
mod lua_value;
pub(crate) mod native_functions;
mod table_value;
mod tuple_value;

pub use function_value::FunctionValue;
pub(crate) use function_value::NativeFunction;
pub use lua_value::*;
pub use table_value::TableValue;
pub use tuple_value::TupleValue;

use crate::nodes::*;

pub(crate) use evaluator_storage::EvaluatorStorage;

/// A struct to convert an Expression node into a LuaValue object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evaluator {
    pure_metamethods: bool,
}

impl Evaluator {
    /// When evaluating expressions related to tables, this value tells the evaluator if
    /// metamethods can have side effects. For example, indexing a normal table in Lua does not
    /// have any side effects, but if the table is a metatable, it's `__index` metamethod can
    /// possibly have side effects (since it can be a function call).
    pub fn assume_pure_metamethods(mut self) -> Self {
        self.pure_metamethods = true;
        self
    }

    pub fn evaluate(&self, expression: &Expression) -> LuaValue {
        let storage = EvaluatorStorage::default();
        self.evaluate_as_tuple(expression, &storage).into_one()
    }

    #[inline]
    pub(crate) fn evaluate_internal(
        &self,
        expression: &Expression,
        storage: &EvaluatorStorage,
    ) -> LuaValue {
        self.evaluate_as_tuple(expression, storage).into_one()
    }

    pub(crate) fn evaluate_as_tuple(
        &self,
        expression: &Expression,
        storage: &EvaluatorStorage,
    ) -> TupleValue {
        let result = match expression {
            Expression::False(_) => LuaValue::False,
            Expression::Function(_) => LuaValue::Function(FunctionValue::new_lua()),
            Expression::Nil(_) => LuaValue::Nil,
            Expression::Number(number) => LuaValue::from(number.compute_value()),
            Expression::String(string) => LuaValue::from(string.get_value()),
            Expression::Table(table) => self.evaluate_table(table, storage),
            Expression::True(_) => LuaValue::True,
            Expression::Binary(binary) => self.evaluate_binary(binary, storage),
            Expression::Unary(unary) => self.evaluate_unary(unary, storage),
            Expression::Parenthese(parenthese) => {
                // when the evaluator will be able to manage tuples, keep only the first element
                // of the tuple here (or coerce the tuple to `nil` if it is empty)
                self.evaluate_as_tuple(parenthese.inner_expression(), storage)
                    .into_one()
            }
            Expression::If(if_expression) => self.evaluate_if(if_expression, storage),
            Expression::InterpolatedString(interpolated_string) => {
                let mut result = Vec::new();
                for segment in interpolated_string.iter_segments() {
                    match segment {
                        InterpolationSegment::String(string) => {
                            result.extend_from_slice(string.get_value());
                        }
                        InterpolationSegment::Value(value) => {
                            match self
                                .evaluate_as_tuple(value.get_expression(), storage)
                                .into_one()
                            {
                                LuaValue::False => {
                                    result.extend_from_slice(b"false");
                                }
                                LuaValue::True => {
                                    result.extend_from_slice(b"true");
                                }
                                LuaValue::Nil => {
                                    result.extend_from_slice(b"nil");
                                }
                                LuaValue::String(string) => {
                                    result.extend_from_slice(&string);
                                }
                                LuaValue::Function(_)
                                | LuaValue::Number(_)
                                | LuaValue::Table(_)
                                | LuaValue::Unknown => return TupleValue::unknown(),
                            }
                        }
                    }
                }
                LuaValue::String(result)
            }
            Expression::TypeCast(type_cast) => self
                .evaluate_as_tuple(type_cast.get_expression(), storage)
                .into_one(),
            Expression::Call(call) => {
                let prefix = self.evaluate_prefix(call.get_prefix(), storage);

                let mut parameters = self.evaluate_arguments(&call.get_arguments(), storage);

                return if let Some(method) = call.get_method() {
                    let function = self.evaluate_table_key(
                        &prefix,
                        LuaValue::from(method.get_name()),
                        storage,
                    );

                    parameters.insert(0, prefix);

                    function.call(parameters)
                } else {
                    prefix.call(parameters)
                };
            }
            Expression::Field(field) => {
                let prefix = self.evaluate_prefix(field.get_prefix(), storage);

                self.evaluate_table_key(
                    &prefix,
                    LuaValue::from(field.get_field().get_name()),
                    storage,
                )
            }
            Expression::Index(index) => {
                let prefix = self.evaluate_prefix(index.get_prefix(), storage);

                self.evaluate_table_key(
                    &prefix,
                    self.evaluate_as_tuple(index.get_index(), storage)
                        .into_one(),
                    storage,
                )
            }
            Expression::Identifier(identifier) => storage.read_identifier(identifier.get_name()),
            Expression::VariableArguments(_) => {
                return TupleValue::unknown();
            }
        };

        TupleValue::known_values(vec![result])
    }

    fn evaluate_table_key(
        &self,
        table: &LuaValue,
        key: LuaValue,
        storage: &EvaluatorStorage,
    ) -> LuaValue {
        if let LuaValue::Table(table_ref) = table {
            storage
                .read_table(*table_ref, |table| table.get(&key))
                .unwrap_or_default()
        } else {
            LuaValue::Unknown
        }
    }

    pub(crate) fn evaluate_prefix(&self, prefix: &Prefix, storage: &EvaluatorStorage) -> LuaValue {
        let mut chain = Vec::new();

        let mut current = prefix;

        let mut result_value = loop {
            let next_prefix = match current {
                Prefix::Call(call) => {
                    if let Some(method) = call.get_method() {
                        chain.push(PrefixOperation::CallMethod(
                            method.get_name(),
                            &call.get_arguments(),
                        ));
                    } else {
                        chain.push(PrefixOperation::Call(&call.get_arguments()));
                    }
                    call.get_prefix()
                }
                Prefix::Field(field) => {
                    chain.push(PrefixOperation::Index(LuaValue::from(
                        field.get_field().get_name(),
                    )));
                    field.get_prefix()
                }
                Prefix::Identifier(identifier) => {
                    break storage.read_identifier(identifier.get_name());
                }
                Prefix::Index(index) => {
                    chain.push(PrefixOperation::IndexDynamic(index.get_index()));
                    index.get_prefix()
                }
                Prefix::Parenthese(parenthese) => {
                    break self
                        .evaluate_as_tuple(parenthese.inner_expression(), storage)
                        .into_one();
                }
            };

            current = next_prefix;
        };

        for op in chain.into_iter().rev() {
            if let LuaValue::Unknown = &result_value {
                break;
            }
            match op {
                PrefixOperation::Call(arguments) => {
                    let parameters = self.evaluate_arguments(arguments, storage);

                    result_value = result_value.call(parameters).into_one();
                }
                PrefixOperation::CallMethod(method, arguments) => {
                    let function =
                        self.evaluate_table_key(&result_value, LuaValue::from(method), storage);

                    let mut parameters = self.evaluate_arguments(arguments, storage);
                    parameters.insert(0, result_value);

                    result_value = function.call(parameters).into_one();
                }
                PrefixOperation::Index(lua_value) => {
                    result_value = self.evaluate_table_key(&result_value, lua_value, storage);
                }
                PrefixOperation::IndexDynamic(index) => {
                    result_value = self.evaluate_table_key(
                        &result_value,
                        self.evaluate_as_tuple(index, storage).into_one(),
                        storage,
                    );
                }
            }
        }

        result_value
    }

    fn evaluate_arguments(&self, arguments: &Arguments, storage: &EvaluatorStorage) -> TupleValue {
        match arguments {
            Arguments::Tuple(tuple) => {
                let length = tuple.len();
                let values: Vec<_> = tuple
                    .iter_values()
                    .take(length.saturating_sub(1))
                    .map(|value| self.evaluate_as_tuple(value, storage).into_one())
                    .collect();

                if length == 0 {
                    TupleValue::empty()
                } else {
                    let mut last_values =
                        self.evaluate_as_tuple(tuple.iter_values().last().unwrap(), storage);
                    last_values.prepend(values);
                    last_values
                }
            }
            Arguments::String(string) => {
                TupleValue::known_values(vec![LuaValue::from(string.get_value())])
            }
            Arguments::Table(table) => {
                TupleValue::known_values(vec![self.evaluate_table(table, storage)])
            }
        }
    }

    fn evaluate_table(&self, _table: &TableExpression, storage: &EvaluatorStorage) -> LuaValue {
        let table = TableValue::new()
            .with_unknown_entries()
            .with_pure_metamethods();

        storage.create_table(table)
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn can_return_multiple_values(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Call(_)
            | Expression::Field(_)
            | Expression::Index(_)
            | Expression::Unary(_)
            | Expression::VariableArguments(_) => true,
            Expression::Binary(binary) => {
                !matches!(binary.operator(), BinaryOperator::And | BinaryOperator::Or)
            }
            Expression::False(_)
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::If(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::Parenthese(_)
            | Expression::String(_)
            | Expression::InterpolatedString(_)
            | Expression::Table(_)
            | Expression::True(_) => false,
            Expression::TypeCast(type_cast) => {
                self.can_return_multiple_values(type_cast.get_expression())
            }
        }
    }

    pub fn has_side_effects(&self, expression: &Expression) -> bool {
        let storage = EvaluatorStorage::default();
        self.has_side_effects_internal(expression, &storage)
    }

    pub(crate) fn has_side_effects_internal(
        &self,
        expression: &Expression,
        storage: &EvaluatorStorage,
    ) -> bool {
        match expression {
            Expression::False(_)
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::True(_)
            | Expression::VariableArguments(_) => false,
            Expression::If(if_expression) => {
                self.if_expression_has_side_effects(if_expression, storage)
            }
            Expression::Binary(binary) => {
                let left = binary.left();
                let right = binary.right();

                let left_value = self.evaluate_internal(left, storage);
                let left_side_effect = self.has_side_effects_internal(binary.left(), storage);

                match binary.operator() {
                    BinaryOperator::And => {
                        if left_value.is_truthy().unwrap_or(true) {
                            left_side_effect
                                || self.has_side_effects_internal(binary.right(), storage)
                        } else {
                            left_side_effect
                        }
                    }
                    BinaryOperator::Or => {
                        if left_value.is_truthy().unwrap_or(false) {
                            left_side_effect
                        } else {
                            left_side_effect
                                || self.has_side_effects_internal(binary.right(), storage)
                        }
                    }
                    _ => {
                        if self.pure_metamethods {
                            left_side_effect
                                || self.has_side_effects_internal(binary.right(), storage)
                        } else {
                            self.maybe_metatable(&left_value)
                                || self.maybe_metatable(&self.evaluate(right))
                                || self.has_side_effects_internal(left, storage)
                                || self.has_side_effects_internal(right, storage)
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
            Expression::Field(field) => self.field_has_side_effects(field, storage),
            Expression::Index(index) => self.index_has_side_effects(index, storage),
            Expression::Parenthese(parenthese) => {
                self.has_side_effects_internal(parenthese.inner_expression(), storage)
            }
            Expression::Table(table) => table
                .get_entries()
                .iter()
                .any(|entry| self.table_entry_has_side_effects(entry, storage)),
            Expression::Call(call) => self.call_has_side_effects(call, storage),
            Expression::InterpolatedString(interpolated_string) => interpolated_string
                .iter_segments()
                .any(|segment| match segment {
                    InterpolationSegment::String(_) => false,
                    InterpolationSegment::Value(value) => {
                        self.has_side_effects_internal(value.get_expression(), storage)
                    }
                }),
            Expression::TypeCast(type_cast) => {
                self.has_side_effects_internal(type_cast.get_expression(), storage)
            }
        }
    }

    fn if_expression_has_side_effects(
        &self,
        if_expression: &IfExpression,
        storage: &EvaluatorStorage,
    ) -> bool {
        if self.has_side_effects_internal(if_expression.get_condition(), storage) {
            return true;
        }

        let condition = self.evaluate_internal(if_expression.get_condition(), storage);

        if let Some(truthy) = condition.is_truthy() {
            if truthy {
                self.has_side_effects_internal(if_expression.get_result(), storage)
            } else {
                for branch in if_expression.iter_branches() {
                    if self.has_side_effects_internal(branch.get_condition(), storage) {
                        return true;
                    }

                    let branch_condition = self.evaluate_internal(branch.get_condition(), storage);

                    if let Some(truthy) = branch_condition.is_truthy() {
                        if truthy {
                            return self.has_side_effects_internal(branch.get_result(), storage);
                        }
                    } else if self.has_side_effects_internal(branch.get_result(), storage) {
                        return true;
                    }
                }

                self.has_side_effects_internal(if_expression.get_else_result(), storage)
            }
        } else {
            if self.has_side_effects_internal(if_expression.get_result(), storage) {
                return true;
            }

            for branch in if_expression.iter_branches() {
                if self.has_side_effects_internal(branch.get_condition(), storage)
                    || self.has_side_effects_internal(branch.get_result(), storage)
                {
                    return true;
                }
            }

            self.has_side_effects_internal(if_expression.get_else_result(), storage)
        }
    }

    #[inline]
    fn call_has_side_effects(&self, call: &FunctionCall, storage: &EvaluatorStorage) -> bool {
        if self.prefix_has_side_effects(call.get_prefix(), storage) {
            return true;
        }
        if call.has_method() && self.table_has_side_effects(call.get_prefix(), storage) {
            return true;
        }

        let prefix = self.evaluate_prefix(call.get_prefix(), storage);

        if let LuaValue::Function(function) = prefix {
            function.has_side_effects()
        } else {
            true
        }
    }

    #[inline]
    fn table_entry_has_side_effects(&self, entry: &TableEntry, storage: &EvaluatorStorage) -> bool {
        match entry {
            TableEntry::Field(entry) => self.has_side_effects_internal(entry.get_value(), storage),
            TableEntry::Index(entry) => {
                self.has_side_effects_internal(entry.get_key(), storage)
                    || self.has_side_effects_internal(entry.get_value(), storage)
            }
            TableEntry::Value(value) => self.has_side_effects_internal(value, storage),
        }
    }

    #[inline]
    fn field_has_side_effects(&self, field: &FieldExpression, storage: &EvaluatorStorage) -> bool {
        let yo = self.table_has_side_effects(field.get_prefix(), storage);
        println!("field_has_side_effects: {:?} = {}", field.get_prefix(), yo);

        yo || self.prefix_has_side_effects(field.get_prefix(), storage)
    }

    fn table_has_side_effects(&self, prefix: &Prefix, storage: &EvaluatorStorage) -> bool {
        println!("table_has_side_effects: {:?}", prefix);
        if self.pure_metamethods {
            false
        } else {
            let r = self.evaluate_prefix(prefix, storage);
            println!("evaluate prefix: {:?}", r);
            if let LuaValue::Table(table_ref) = r {
                println!("  found table");

                storage
                    .read_table(table_ref, |table| !table.has_pure_metamethods())
                    .unwrap_or(true)
            } else {
                println!("  no table");
                true
            }
        }
    }

    #[inline]
    fn index_has_side_effects(&self, index: &IndexExpression, storage: &EvaluatorStorage) -> bool {
        self.table_has_side_effects(index.get_prefix(), storage)
            || self.has_side_effects_internal(index.get_index(), storage)
    }

    fn prefix_has_side_effects(&self, prefix: &Prefix, storage: &EvaluatorStorage) -> bool {
        println!("prefix_has_side_effects: {:?}", prefix);
        match prefix {
            Prefix::Call(call) => self.call_has_side_effects(call, storage),
            Prefix::Field(field) => self.field_has_side_effects(field, storage),
            Prefix::Identifier(_) => false,
            Prefix::Index(index) => self.index_has_side_effects(index, storage),
            Prefix::Parenthese(sub_expression) => {
                self.has_side_effects_internal(sub_expression.inner_expression(), storage)
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
            | LuaValue::Table(_)
            | LuaValue::True => false,
            LuaValue::Unknown => true,
        }
    }

    fn evaluate_binary(
        &self,
        expression: &BinaryExpression,
        storage: &EvaluatorStorage,
    ) -> LuaValue {
        match expression.operator() {
            BinaryOperator::And => self
                .evaluate_internal(expression.left(), storage)
                .map_if_truthy(|_| self.evaluate_internal(expression.right(), storage)),
            BinaryOperator::Or => self
                .evaluate_internal(expression.left(), storage)
                .map_if_truthy_else(
                    |left| left,
                    || self.evaluate_internal(expression.right(), storage),
                ),
            BinaryOperator::Equal => self.evaluate_equal(
                &self.evaluate_internal(expression.left(), storage),
                &self.evaluate_internal(expression.right(), storage),
            ),
            BinaryOperator::NotEqual => {
                let result = self.evaluate_equal(
                    &self.evaluate_internal(expression.left(), storage),
                    &self.evaluate_internal(expression.right(), storage),
                );

                match result {
                    LuaValue::True => LuaValue::False,
                    LuaValue::False => LuaValue::True,
                    _ => LuaValue::Unknown,
                }
            }
            BinaryOperator::Plus => self.evaluate_math(expression, |a, b| a + b, storage),
            BinaryOperator::Minus => self.evaluate_math(expression, |a, b| a - b, storage),
            BinaryOperator::Asterisk => self.evaluate_math(expression, |a, b| a * b, storage),
            BinaryOperator::Slash => self.evaluate_math(expression, |a, b| a / b, storage),
            BinaryOperator::DoubleSlash => {
                self.evaluate_math(expression, |a, b| (a / b).floor(), storage)
            }
            BinaryOperator::Caret => self.evaluate_math(expression, |a, b| a.powf(b), storage),
            BinaryOperator::Percent => {
                self.evaluate_math(expression, |a, b| a - b * (a / b).floor(), storage)
            }
            BinaryOperator::Concat => {
                match (
                    self.evaluate_internal(expression.left(), storage)
                        .string_coercion(),
                    self.evaluate_internal(expression.right(), storage)
                        .string_coercion(),
                ) {
                    (LuaValue::String(mut left), LuaValue::String(right)) => {
                        left.extend_from_slice(&right);
                        LuaValue::String(left)
                    }
                    _ => LuaValue::Unknown,
                }
            }
            BinaryOperator::LowerThan => {
                self.evaluate_relational(expression, |a, b| a < b, storage)
            }
            BinaryOperator::LowerOrEqualThan => {
                self.evaluate_relational(expression, |a, b| a <= b, storage)
            }
            BinaryOperator::GreaterThan => {
                self.evaluate_relational(expression, |a, b| a > b, storage)
            }
            BinaryOperator::GreaterOrEqualThan => {
                self.evaluate_relational(expression, |a, b| a >= b, storage)
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

    fn evaluate_math<F>(
        &self,
        expression: &BinaryExpression,
        operation: F,
        storage: &EvaluatorStorage,
    ) -> LuaValue
    where
        F: Fn(f64, f64) -> f64,
    {
        let left = self
            .evaluate_internal(expression.left(), storage)
            .number_coercion();

        if let LuaValue::Number(left) = left {
            let right = self
                .evaluate_internal(expression.right(), storage)
                .number_coercion();

            if let LuaValue::Number(right) = right {
                LuaValue::Number(operation(left, right))
            } else {
                LuaValue::Unknown
            }
        } else {
            LuaValue::Unknown
        }
    }

    fn evaluate_relational<F>(
        &self,
        expression: &BinaryExpression,
        operation: F,
        storage: &EvaluatorStorage,
    ) -> LuaValue
    where
        F: Fn(f64, f64) -> bool,
    {
        let left = self.evaluate_internal(expression.left(), storage);

        match left {
            LuaValue::Number(left) => {
                let right = self.evaluate_internal(expression.right(), storage);

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
                let right = self.evaluate_internal(expression.right(), storage);

                if let LuaValue::String(right) = right {
                    self.compare_strings(&left, &right, expression.operator())
                } else {
                    LuaValue::Unknown
                }
            }
            _ => LuaValue::Unknown,
        }
    }

    fn compare_strings(&self, left: &[u8], right: &[u8], operator: BinaryOperator) -> LuaValue {
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

    fn evaluate_unary(&self, expression: &UnaryExpression, storage: &EvaluatorStorage) -> LuaValue {
        match expression.operator() {
            UnaryOperator::Not => self
                .evaluate_internal(expression.get_expression(), storage)
                .is_truthy()
                .map(|value| LuaValue::from(!value))
                .unwrap_or(LuaValue::Unknown),
            UnaryOperator::Minus => {
                match self
                    .evaluate_internal(expression.get_expression(), storage)
                    .number_coercion()
                {
                    LuaValue::Number(value) => LuaValue::from(-value),
                    _ => LuaValue::Unknown,
                }
            }
            UnaryOperator::Length => self
                .evaluate_internal(expression.get_expression(), storage)
                .length(),
        }
    }

    fn evaluate_if(&self, expression: &IfExpression, storage: &EvaluatorStorage) -> LuaValue {
        let condition = self.evaluate_internal(expression.get_condition(), storage);

        if let Some(truthy) = condition.is_truthy() {
            if truthy {
                self.evaluate_internal(expression.get_result(), storage)
            } else {
                for branch in expression.iter_branches() {
                    let branch_condition = self.evaluate_internal(branch.get_condition(), storage);
                    if let Some(truthy) = branch_condition.is_truthy() {
                        if truthy {
                            return self.evaluate_internal(branch.get_result(), storage);
                        }
                    } else {
                        return LuaValue::Unknown;
                    }
                }

                self.evaluate_internal(expression.get_else_result(), storage)
            }
        } else {
            LuaValue::Unknown
        }
    }
}

enum PrefixOperation<'a> {
    Call(&'a Arguments),
    CallMethod(&'a str, &'a Arguments),
    Index(LuaValue),
    IndexDynamic(&'a Expression),
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
        number_expression_negative_zero(DecimalNumber::new(-0.0)) => LuaValue::Number(-0.0),
        string_expression(StringExpression::from_value("foo")) => LuaValue::from("foo"),
        empty_interpolated_string_expression(InterpolatedStringExpression::empty()) => LuaValue::from(""),
        interpolated_string_expression_with_one_string(InterpolatedStringExpression::empty().with_segment("hello"))
            => LuaValue::from("hello"),
        interpolated_string_expression_with_multiple_string_segments(
            InterpolatedStringExpression::empty()
                .with_segment("hello")
                .with_segment("-")
                .with_segment("bye")
        ) => LuaValue::from("hello-bye"),
        interpolated_string_expression_with_true_segment(
            InterpolatedStringExpression::empty().with_segment(Expression::from(true))
        ) => LuaValue::from("true"),
        interpolated_string_expression_with_false_segment(
            InterpolatedStringExpression::empty().with_segment(Expression::from(false))
        ) => LuaValue::from("false"),
        interpolated_string_expression_with_nil_segment(
            InterpolatedStringExpression::empty().with_segment(Expression::nil())
        ) => LuaValue::from("nil"),
        interpolated_string_expression_with_mixed_segments(
            InterpolatedStringExpression::empty()
                .with_segment("variable = ")
                .with_segment(Expression::from(true))
                .with_segment("?")
        ) => LuaValue::from("variable = true?"),
        interpolated_string_expression_with_mixed_segments_unknown(
            InterpolatedStringExpression::empty()
                .with_segment("variable = ")
                .with_segment(Expression::identifier("test"))
                .with_segment("!")
        ) => LuaValue::Unknown,
        true_wrapped_in_parens(ParentheseExpression::new(true)) => LuaValue::True,
        false_wrapped_in_parens(ParentheseExpression::new(false)) => LuaValue::False,
        nil_wrapped_in_parens(ParentheseExpression::new(Expression::nil())) => LuaValue::Nil,
        number_wrapped_in_parens(ParentheseExpression::new(DecimalNumber::new(0.0)))
            => LuaValue::Number(0.0),
        string_wrapped_in_parens(ParentheseExpression::new(StringExpression::from_value("foo")))
            => LuaValue::from("foo"),
        // table_expression(TableExpression::default()) => LuaValue::Table,
        if_expression_always_true(IfExpression::new(true, 1.0, 0.0)) => LuaValue::from(1.0),
        if_expression_always_false(IfExpression::new(false, 1.0, 0.0)) => LuaValue::from(0.0),
        if_expression_unknown_condition(IfExpression::new(Expression::identifier("test"), 1.0, 0.0))
            => LuaValue::Unknown,
        if_expression_elseif_always_true(IfExpression::new(false, 1.0, 0.0).with_branch(true, 2.0))
            => LuaValue::from(2.0),
        if_expression_elseif_always_false(IfExpression::new(false, 1.0, 0.0).with_branch(false, 2.0))
            => LuaValue::from(0.0),
        length_empty_string(UnaryExpression::new(UnaryOperator::Length, StringExpression::empty()))
            => LuaValue::Number(0.0),
        length_single_char_string(UnaryExpression::new(UnaryOperator::Length, StringExpression::from_value("a")))
            => LuaValue::Number(1.0),
        length_short_string(UnaryExpression::new(UnaryOperator::Length, StringExpression::from_value("hello")))
            => LuaValue::Number(5.0),
        length_unknown_expression(UnaryExpression::new(UnaryOperator::Length, Expression::identifier("var")))
            => LuaValue::Unknown,
        length_number_expression(UnaryExpression::new(UnaryOperator::Length, Expression::from(42.0)))
            => LuaValue::Unknown,
        length_nil_expression(UnaryExpression::new(UnaryOperator::Length, Expression::nil()))
            => LuaValue::Unknown,
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

                        match (&$expect, &result) {
                            (LuaValue::Number(expect_float), LuaValue::Number(result)) => {
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
                                    assert!(
                                        expect_float.is_sign_positive() == result.is_sign_positive(),
                                        "{} should be of the same sign as {}", result, expect_float
                                    );
                                }
                            }
                            _ => {
                                assert_eq!($expect, result);
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
            ) => LuaValue::from("foo"),
            // true_and_table(
            //     BinaryOperator::And,
            //     true,
            //     TableExpression::default()
            // ) => LuaValue::Table,
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
            ) => LuaValue::Number(f64::INFINITY),
            negative_zero_plus_negative_zero(
                BinaryOperator::Plus,
                Expression::from(-0.0),
                Expression::from(-0.0)
            ) => LuaValue::Number(-0.0),
            negative_zero_minus_zero(
                BinaryOperator::Minus,
                Expression::from(-0.0),
                Expression::from(0.0)
            ) => LuaValue::Number(-0.0),
            zero_divided_by_zero(
                BinaryOperator::Slash,
                Expression::from(0.0),
                Expression::from(0.0)
            ) => LuaValue::Number(f64::NAN),
            twelve_floor_division_by_four(
                BinaryOperator::DoubleSlash,
                Expression::from(12.0),
                Expression::from(4.0)
            ) => LuaValue::Number(3.0),
            eleven_floor_division_by_three(
                BinaryOperator::DoubleSlash,
                Expression::from(11.0),
                Expression::from(3.0)
            ) => LuaValue::Number(3.0),
            one_floor_division_by_zero(
                BinaryOperator::DoubleSlash,
                Expression::from(1.0),
                Expression::from(0.0)
            ) => LuaValue::Number(f64::INFINITY),
            minus_one_floor_division_by_zero(
                BinaryOperator::DoubleSlash,
                Expression::from(-1.0),
                Expression::from(0.0)
            ) => LuaValue::Number(f64::NEG_INFINITY),
            zero_floor_division_by_zero(
                BinaryOperator::DoubleSlash,
                Expression::from(0.0),
                Expression::from(0.0)
            ) => LuaValue::Number(f64::NAN),
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
            minus_zero(Minus, DecimalNumber::new(-0.0)) => LuaValue::from(-0.0),
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

        interpolated_string_with_function_call => InterpolatedStringExpression::empty()
            .with_segment(Expression::from(FunctionCall::from_name("foo"))),
    );

    has_no_side_effects!(
        true_value => Expression::from(true),
        false_value => Expression::from(false),
        nil_value => Expression::nil(),
        table_value => TableExpression::default(),
        number_value => Expression::Number(DecimalNumber::new(0.0).into()),
        string_value => StringExpression::from_value(""),
        empty_interpolated_string_value => InterpolatedStringExpression::empty(),
        interpolated_string_with_true_value => InterpolatedStringExpression::empty()
            .with_segment(Expression::from(true)),
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
