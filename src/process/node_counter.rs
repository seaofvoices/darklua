use crate::nodes::*;
use crate::process::NodeProcessor;

pub struct NodeCounter {
    pub block_count: usize,
    pub function_call_count: usize,
    pub assign_count: usize,
    pub do_count: usize,
    pub function_count: usize,
    pub generic_for_count: usize,
    pub if_count: usize,
    pub local_assign_count: usize,
    pub local_function_count: usize,
    pub numeric_for_count: usize,
    pub repeat_count: usize,
    pub while_count: usize,
    pub break_count: usize,
    pub return_count: usize,
    pub expression_count: usize,
}

impl NodeCounter {
    pub fn new() -> Self {
        Self {
            block_count: 0,
            function_call_count: 0,
            assign_count: 0,
            do_count: 0,
            function_count: 0,
            generic_for_count: 0,
            if_count: 0,
            local_assign_count: 0,
            local_function_count: 0,
            numeric_for_count: 0,
            repeat_count: 0,
            while_count: 0,
            break_count: 0,
            return_count: 0,
            expression_count: 0,
        }
    }
}

impl NodeProcessor for NodeCounter {
    fn process_block(&mut self, _: &mut Block) {
        self.block_count += 1;
    }

    fn process_function_call(&mut self, _: &mut FunctionCall) {
        self.function_call_count += 1;
    }

    fn process_assign_statement(&mut self, _: &mut AssignStatement) {
        self.assign_count += 1;
    }

    fn process_do_statement(&mut self, _: &mut DoStatement) {
        self.do_count += 1;
    }

    fn process_function_statement(&mut self, _: &mut FunctionStatement) {
        self.function_count += 1;
    }

    fn process_generic_for_statement(&mut self, _: &mut GenericForStatement) {
        self.generic_for_count += 1;
    }

    fn process_if_statement(&mut self, _: &mut IfStatement) {
        self.if_count += 1;
    }

    fn process_last_statement(&mut self, statement: &mut LastStatement) {
        match statement {
            LastStatement::Break => self.break_count += 1,
            LastStatement::Return(_) => self.return_count += 1,
        }
    }

    fn process_local_assign_statement(&mut self, _: &mut LocalAssignStatement) {
        self.local_assign_count += 1;
    }

    fn process_local_function_statement(&mut self, _: &mut LocalFunctionStatement) {
        self.local_function_count += 1;
    }

    fn process_numeric_for_statement(&mut self, _: &mut NumericForStatement) {
        self.numeric_for_count += 1;
    }

    fn process_repeat_statement(&mut self, _: &mut RepeatStatement) {
        self.repeat_count += 1;
    }

    fn process_while_statement(&mut self, _: &mut WhileStatement) {
        self.while_count += 1;
    }

    fn process_expression(&mut self, _: &mut Expression) {
        self.expression_count += 1;
    }
}
