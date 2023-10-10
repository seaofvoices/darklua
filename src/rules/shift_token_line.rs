use crate::nodes::*;
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

#[derive(Debug)]
struct Processor {
    shift_amount: usize,
}

impl Processor {
    fn new(shift_amount: usize) -> Self {
        Self { shift_amount }
    }
}

impl NodeProcessor for Processor {
    fn process_block(&mut self, block: &mut Block) {
        block.shift_token_line(self.shift_amount);
    }

    fn process_function_call(&mut self, call: &mut FunctionCall) {
        call.shift_token_line(self.shift_amount);
        call.mutate_arguments().shift_token_line(self.shift_amount);
    }

    fn process_assign_statement(&mut self, assign: &mut AssignStatement) {
        assign.shift_token_line(self.shift_amount);
    }

    fn process_compound_assign_statement(&mut self, assign: &mut CompoundAssignStatement) {
        assign.shift_token_line(self.shift_amount);
    }

    fn process_do_statement(&mut self, statement: &mut DoStatement) {
        statement.shift_token_line(self.shift_amount);
    }

    fn process_function_statement(&mut self, function: &mut FunctionStatement) {
        function.shift_token_line(self.shift_amount);
    }

    fn process_generic_for_statement(&mut self, generic_for: &mut GenericForStatement) {
        generic_for.shift_token_line(self.shift_amount);
    }

    fn process_if_statement(&mut self, if_statement: &mut IfStatement) {
        if_statement.shift_token_line(self.shift_amount);
    }

    fn process_last_statement(&mut self, statement: &mut LastStatement) {
        match statement {
            LastStatement::Break(token) | LastStatement::Continue(token) => {
                if let Some(token) = token {
                    token.shift_token_line(self.shift_amount);
                }
            }
            LastStatement::Return(statement) => statement.shift_token_line(self.shift_amount),
        }
    }

    fn process_local_assign_statement(&mut self, assign: &mut LocalAssignStatement) {
        assign.shift_token_line(self.shift_amount);
    }

    fn process_local_function_statement(&mut self, function: &mut LocalFunctionStatement) {
        function.shift_token_line(self.shift_amount);
    }

    fn process_numeric_for_statement(&mut self, numeric_for: &mut NumericForStatement) {
        numeric_for.shift_token_line(self.shift_amount);
    }

    fn process_repeat_statement(&mut self, repeat: &mut RepeatStatement) {
        repeat.shift_token_line(self.shift_amount);
    }

    fn process_while_statement(&mut self, statement: &mut WhileStatement) {
        statement.shift_token_line(self.shift_amount);
    }

    fn process_expression(&mut self, expression: &mut Expression) {
        match expression {
            Expression::False(token)
            | Expression::Nil(token)
            | Expression::True(token)
            | Expression::VariableArguments(token) => {
                if let Some(token) = token {
                    token.shift_token_line(self.shift_amount)
                }
            }
            Expression::Binary(_)
            | Expression::Call(_)
            | Expression::Field(_)
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::If(_)
            | Expression::Index(_)
            | Expression::Number(_)
            | Expression::Parenthese(_)
            | Expression::String(_)
            | Expression::Table(_)
            | Expression::Unary(_)
            | Expression::TypeCast(_) => {}
        }
    }

    fn process_binary_expression(&mut self, binary: &mut BinaryExpression) {
        binary.shift_token_line(self.shift_amount);
    }

    fn process_field_expression(&mut self, field: &mut FieldExpression) {
        field.shift_token_line(self.shift_amount);
    }

    fn process_function_expression(&mut self, function: &mut FunctionExpression) {
        function.shift_token_line(self.shift_amount);
    }

    fn process_if_expression(&mut self, if_expression: &mut IfExpression) {
        if_expression.shift_token_line(self.shift_amount);
    }

    fn process_variable_expression(&mut self, identifier: &mut Identifier) {
        identifier.shift_token_line(self.shift_amount);
    }

    fn process_index_expression(&mut self, index: &mut IndexExpression) {
        index.shift_token_line(self.shift_amount);
    }

    fn process_number_expression(&mut self, number: &mut NumberExpression) {
        number.shift_token_line(self.shift_amount);
    }

    fn process_parenthese_expression(&mut self, expression: &mut ParentheseExpression) {
        expression.shift_token_line(self.shift_amount);
    }

    fn process_string_expression(&mut self, string: &mut StringExpression) {
        string.shift_token_line(self.shift_amount);
    }

    fn process_table_expression(&mut self, table: &mut TableExpression) {
        table.shift_token_line(self.shift_amount);
    }

    fn process_unary_expression(&mut self, unary: &mut UnaryExpression) {
        unary.shift_token_line(self.shift_amount);
    }

    fn process_prefix_expression(&mut self, _: &mut Prefix) {}

    fn process_type(&mut self, r#type: &mut Type) {
        r#type.shift_token_line(self.shift_amount);
    }
}

pub const SHIFT_TOKEN_LINE: &str = "shift_token_line";

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ShiftTokenLine {
    shift_amount: usize,
}

impl ShiftTokenLine {
    pub(crate) fn new(shift_amount: usize) -> Self {
        Self { shift_amount }
    }
}

impl FlawlessRule for ShiftTokenLine {
    fn flawless_process(&self, block: &mut Block, _context: &Context) {
        if self.shift_amount != 0 {
            let mut processor = Processor::new(self.shift_amount);
            DefaultVisitor::visit_block(block, &mut processor);
        }
    }
}

impl RuleConfiguration for ShiftTokenLine {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        SHIFT_TOKEN_LINE
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> ShiftTokenLine {
        ShiftTokenLine::new(1)
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_replace_referenced_tokens", rule);
    }
}
