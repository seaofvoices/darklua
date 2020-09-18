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
    fn process_block(&mut self, _: &Block) {
        self.block_count += 1;
    }

    fn process_function_call(&mut self, _: &FunctionCall) {
        self.function_call_count += 1;
    }

    fn process_assign_statement(&mut self, _: &AssignStatement) {
        self.assign_count += 1;
    }

    fn process_do_statement(&mut self, _: &DoStatement) {
        self.do_count += 1;
    }

    fn process_function_statement(&mut self, _: &FunctionStatement) {
        self.function_count += 1;
    }

    fn process_generic_for_statement(&mut self, _: &GenericForStatement) {
        self.generic_for_count += 1;
    }

    fn process_if_statement(&mut self, _: &IfStatement) {
        self.if_count += 1;
    }

    fn process_last_statement(&mut self, statement: &LastStatement) {
        match statement {
            LastStatement::Break => self.break_count += 1,
            LastStatement::Return(_) => self.return_count += 1,
        }
    }

    fn process_local_assign_statement(&mut self, _: &LocalAssignStatement) {
        self.local_assign_count += 1;
    }

    fn process_local_function_statement(&mut self, _: &LocalFunctionStatement) {
        self.local_function_count += 1;
    }

    fn process_numeric_for_statement(&mut self, _: &NumericForStatement) {
        self.numeric_for_count += 1;
    }

    fn process_repeat_statement(&mut self, _: &RepeatStatement) {
        self.repeat_count += 1;
    }

    fn process_while_statement(&mut self, _: &WhileStatement) {
        self.while_count += 1;
    }

    fn process_expression(&mut self, _: &Expression) {
        self.expression_count += 1;
    }
}
