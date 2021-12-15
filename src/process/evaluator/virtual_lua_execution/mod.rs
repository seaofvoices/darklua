mod execution_effect;
mod local_variable;
mod state;

use crate::nodes::*;

use execution_effect::ExecutionEffect;
use state::State;

use super::{Evaluator, LuaValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationResult {
    None,
    Return(Vec<LuaValue>),
    Break,
    Continue,
}

impl EvaluationResult {
    #[inline]
    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualLuaExecution {
    states: Vec<State>,
    current: usize,
    effects: ExecutionEffect,
    max_loop_iteration: usize,
}

impl Default for VirtualLuaExecution {
    fn default() -> Self {
        Self {
            states: vec![State::new_root(0)],
            current: 0,
            effects: ExecutionEffect::default(),
            max_loop_iteration: 500,
        }
    }
}

impl VirtualLuaExecution {
    pub fn with_global_value<S: Into<String>>(mut self, name: S, value: LuaValue) -> Self {
        self.current_state_mut().insert_local(name, value);
        self
    }

    pub fn evaluate_function(&mut self, block: &Block) -> Option<Vec<LuaValue>> {
        match self.process_block(block) {
            EvaluationResult::Return(value) => Some(value),
            EvaluationResult::None | EvaluationResult::Break | EvaluationResult::Continue => None,
        }
    }

    fn fork_state(&mut self) -> usize {
        let parent = self.current;
        let id = self.states.len();
        self.states.push(State::new(id, parent));
        self.current = id;
        parent
    }

    pub fn process(&mut self, statement: &Statement) -> EvaluationResult {
        match statement {
            Statement::Assign(assign) => {
                let values = assign.get_values();
                for (i, variable) in assign.iter_variables().enumerate() {
                    match variable {
                        Variable::Identifier(identifier) => {
                            let value = values
                                .get(i)
                                .map(|expression| self.evaluate_expression(expression))
                                .unwrap_or(LuaValue::Nil);
                            let identifier_value = identifier.get_name();

                            if let Some(state) = self
                                .find_ancestor_with_identifier(identifier_value)
                                .and_then(|id| self.mut_state(id))
                            {
                                state.assign_identifier(identifier_value, value);
                                self.effects.add(identifier_value);
                            }
                        }
                        Variable::Field(_) => todo!(),
                        Variable::Index(_) => todo!(),
                    }
                }
                EvaluationResult::None
            }
            Statement::Do(do_statement) => self.process_block(do_statement.get_block()),
            Statement::Call(_) => todo!(),
            Statement::CompoundAssign(_) => todo!(),
            Statement::Function(_) => todo!(),
            Statement::GenericFor(_) => todo!(),
            Statement::If(if_statement) => {
                let mut else_should_run = Some(true);
                for branch in if_statement.get_branches() {
                    match self.evaluate_expression(branch.get_condition()).is_truthy() {
                        Some(true) => {
                            if else_should_run.is_some() {
                                let result = self.process_block(branch.get_block());
                                if !result.is_none() {
                                    return result;
                                }
                                else_should_run = Some(false);
                            } else {
                                self.process_conditional_block(branch.get_block());
                            }
                            break;
                        }
                        Some(false) => continue,
                        None => {
                            // this branch may run, so we need to blur all locals it captures
                            self.process_conditional_block(branch.get_block());
                            else_should_run = None;
                        }
                    }
                }

                if let Some(else_block) = if_statement.get_else_block() {
                    match else_should_run {
                        Some(false) => {}
                        Some(true) => return self.process_block(else_block),
                        None => {
                            self.process_conditional_block(else_block);
                        }
                    }
                }

                EvaluationResult::None
            }
            Statement::LocalAssign(assign) => {
                let values = assign.get_values();
                for (i, identifier) in assign.iter_variables().enumerate() {
                    let value = values
                        .get(i)
                        .map(|expression| self.evaluate_expression(expression))
                        .unwrap_or(LuaValue::Nil);
                    self.current_state_mut()
                        .insert_local(identifier.get_name(), value);
                }

                EvaluationResult::None
            }
            Statement::LocalFunction(function) => {
                let name = function.get_name();
                self.current_state_mut()
                    .insert_local(name, LuaValue::Function);

                EvaluationResult::None
            }
            Statement::NumericFor(for_statement) => {
                let init = self.evaluate_expression(for_statement.get_start());
                let end = self.evaluate_expression(for_statement.get_end());
                let step = for_statement
                    .get_step()
                    .map(|step| self.evaluate_expression(step))
                    .unwrap_or_else(|| LuaValue::from(1.0));

                if let (LuaValue::Number(init), LuaValue::Number(end), LuaValue::Number(step)) =
                    (init, end, step)
                {
                    if (step == 0.0) || (step > 0.0 && init > end) || (step < 0.0 && init < end) {
                        return EvaluationResult::None;
                    }

                    let mut iteration = 0;
                    let mut variable = init;
                    let parent_id = self.fork_state();

                    let for_loop_state_id = self.current_state().id();
                    let variable_name = for_statement.get_identifier().get_name();

                    let result = loop {
                        if iteration >= self.max_loop_iteration {
                            self.process_conditional_block(for_statement.get_block());
                            break EvaluationResult::None;
                        }
                        if (step > 0.0 && variable > end) || (step < 0.0 && variable < end) {
                            break EvaluationResult::None;
                        }

                        self.mut_state(for_loop_state_id)
                            .expect("for loop state should exist")
                            .insert_local(variable_name, LuaValue::Number(variable));

                        let result = self.process_block(for_statement.get_block());
                        match result {
                            EvaluationResult::Return(_) => return result,
                            EvaluationResult::Break => {
                                break EvaluationResult::None;
                            }
                            EvaluationResult::None | EvaluationResult::Continue => {}
                        }
                        variable += step;
                        iteration += 1;
                    };

                    self.current = parent_id;
                    result
                } else {
                    self.process_conditional_block(for_statement.get_block());
                    EvaluationResult::None
                }
            }
            Statement::Repeat(repeat) => {
                let mut iteration = 0;
                loop {
                    if iteration >= self.max_loop_iteration {
                        self.process_conditional_block(repeat.get_block());
                        // TODO process condition expression
                        break EvaluationResult::None;
                    }
                    let result = self.process_block(repeat.get_block());
                    match result {
                        EvaluationResult::Return(_) => return result,
                        EvaluationResult::Break => {
                            break EvaluationResult::None;
                        }
                        EvaluationResult::None | EvaluationResult::Continue => {}
                    }
                    match self.evaluate_expression(repeat.get_condition()).is_truthy() {
                        Some(false) => {}
                        Some(true) => break EvaluationResult::None,
                        None => {
                            self.process_conditional_block(repeat.get_block());
                            break EvaluationResult::None;
                        }
                    }
                    iteration += 1;
                }
            }
            Statement::While(while_statement) => {
                let mut iteration = 0;
                loop {
                    if iteration >= self.max_loop_iteration {
                        // TODO process condition expression
                        self.process_conditional_block(while_statement.get_block());
                        break EvaluationResult::None;
                    }
                    match self
                        .evaluate_expression(while_statement.get_condition())
                        .is_truthy()
                    {
                        Some(true) => {
                            let result = self.process_block(while_statement.get_block());
                            match result {
                                EvaluationResult::Return(_) => return result,
                                EvaluationResult::Break => {
                                    break EvaluationResult::None;
                                }
                                EvaluationResult::None | EvaluationResult::Continue => {}
                            }
                        }
                        Some(false) => {
                            break EvaluationResult::None;
                        }
                        None => {
                            self.process_conditional_block(while_statement.get_block());
                            break EvaluationResult::None;
                        }
                    }
                    iteration += 1;
                }
            }
        }
    }

    fn process_block(&mut self, block: &Block) -> EvaluationResult {
        let parent_id = self.fork_state();

        for statement in block.iter_statements() {
            let result = self.process(statement);

            match result {
                EvaluationResult::Return(_)
                | EvaluationResult::Break
                | EvaluationResult::Continue => return result,
                EvaluationResult::None => {}
            }
        }

        let result = if let Some(last) = block.get_last_statement() {
            self.process_last_statement(last)
        } else {
            EvaluationResult::None
        };

        self.current = parent_id;
        result
    }

    fn process_last_statement(&mut self, statement: &LastStatement) -> EvaluationResult {
        match statement {
            LastStatement::Break(_) => EvaluationResult::Break,
            LastStatement::Continue(_) => EvaluationResult::Continue,
            LastStatement::Return(statement) => EvaluationResult::Return(
                statement
                    .iter_expressions()
                    .map(|expression| self.evaluate_expression(expression))
                    .collect(),
            ),
        }
    }

    fn process_conditional_block(&mut self, block: &Block) {
        self.effects.enable();
        self.process_block(block);

        for identifier in self.effects.disable() {
            if let Some(state) = self
                .find_ancestor_with_identifier(&identifier)
                .and_then(|id| self.mut_state(id))
            {
                state.assign_identifier(&identifier, LuaValue::Unknown);
            }
        }
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> LuaValue {
        let evaluator = Evaluator::new(self);
        evaluator.evaluate(expression)
        // TODO: iterate through expression to find potential side effect that needs
        // to blur variables (like function calls, index or field expressions)
        // TODO: potentially replace expression here since they can be evaluated
    }

    fn current_state_mut(&mut self) -> &mut State {
        self.states
            .get_mut(self.current)
            .expect("current state should always exist")
    }

    fn current_state(&self) -> &State {
        self.states
            .get(self.current)
            .expect("current state should always exist")
    }

    #[inline]
    fn get_state(&self, id: usize) -> Option<&State> {
        self.states.get(id)
    }

    #[inline]
    fn mut_state(&mut self, id: usize) -> Option<&mut State> {
        self.states.get_mut(id)
    }

    pub(crate) fn evaluate_identifier(&self, name: &str) -> LuaValue {
        self.current_state()
            .read(name, self)
            .unwrap_or(LuaValue::Unknown)
    }

    fn find_ancestor_with_identifier(&self, identifier: &str) -> Option<usize> {
        let mut current = self.current_state();
        while !current.has_identifier(identifier) {
            current = self.get_state(current.parent()?)?;
        }
        Some(current.id())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! evaluate_block {
        ($($name:ident ($code:literal) => [$( $result:expr ),*] ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let block = crate::Parser::default()
                        .parse($code)
                        .expect("code should parse");

                    let mut state = VirtualLuaExecution::default();

                    pretty_assertions::assert_eq!(
                        state.evaluate_function(&block),
                        Some(vec![$( LuaValue::from($result), )*])
                    );
                }
            )*
        };
    }

    evaluate_block!(
        return_nothing("return") => [],
        return_nil("return nil") => [LuaValue::Nil],
        return_true("return true") => [true],
        return_false("return false") => [false],
        return_true_false("return true, false") => [true, false],
        return_created_local("local var = true; return var") => [true],
        return_variable_addition(
            "local number = 1; local amount = 3; return number + amount"
        ) => [4.0],
        return_from_do("local a = 'str' do return a end") => ["str"],
        reassign_value("local var = 1; var = 2; return var + var") => [4.0],
        reassign_value_in_another_block("local var = 1; do var = 2 end return var * var") => [4.0],
        reassign_same_local_does_not_override("local var = 1; do local var = 2 end return var") => [1.0],
        assignment_blurs_variable_in_if_statement(
            "local var = 1; if condition then var = 2 end return var"
        ) => [LuaValue::Unknown],
        assignment_blurs_variable_in_if_statement_else(
            "local var = 1; if condition then return else var = 2 end return var"
        ) => [LuaValue::Unknown],
        assignment_in_if_statement_branch_works_if_branch_is_known(
            "local var = 1; if var == 1 then var = 2 end return var"
        ) => [2.0],
        assignment_in_if_statement_else_branch_works_if_branch_is_known(
            "local var = 1; if var > 10 then var = 10 else var = 0 end return var"
        ) => [0.0],
        assignment_in_if_statement_elseif_branch_works_if_branch_is_known(
            "local var = 1; if var > 10 then var = 10 elseif var > 0 then var = var + var end return var"
        ) => [2.0],
        return_in_if_statement_works_if_branch_is_known(
            "local var = 12; if var > 10 then return 10 else return var end"
        ) => [10.0],
        while_with_false_condition_does_not_blur(
            "local var = false while var do var = nil end return var"
        ) => [false],
        enter_while_with_true_condition_computes(
            "local n = 1 while n < 5 do n = n + 1 end return n"
        ) => [5.0],
        while_with_unknown_condition_blurs_variables(
            "local n = 1 while condition do n = n + 1 end return n"
        ) => [LuaValue::Unknown],
        infinite_while_break_in_if(
            "local n = 1 while true do n = n + 1 if n == 5 then break end end return n"
        ) => [5.0],
        infinite_while_return_in_if(
            "local n = 1 while true do n = n + 1 if n == 5 then return 'ok' end end return n"
        ) => ["ok"],
        infinite_while("local n = 0 while true do n = n + 1 end return n") => [LuaValue::Unknown],
        repeat_runs_once("local n = 1 repeat n = n + 1 until true return n") => [2.0],
        repeat_runs_until_condition_is_true(
            "local n = 1 repeat n = n + 1 until n == 5 return n"
        ) => [5.0],
        repeat_breaks_immediately(
            "local n = 1 repeat break until n == 5 return n"
        ) => [1.0],
        return_from_condition_in_repeat(
            "local n = 1 repeat n = n + 1 if n == 5 then return 'ok' end until false return n"
        ) => ["ok"],
        repeat_with_unknown_condition(
            "local n = 1 repeat n = n + 1 until condition return n"
        ) => [LuaValue::Unknown],
        infinite_repeat("local n = 1 repeat n = n + 1 until false return n") => [LuaValue::Unknown],
        numeric_for_with_known_bounds(
            "local n = 0 for i = 1, 10 do n = n + i end return n"
        ) => [55.0],
        numeric_for_with_step_equal_to_zero_does_not_run(
            "local n = 0 for i = 1, 10, 0 do n = n + i end return n"
        ) => [0.0],
        numeric_for_breaks_in_if(
            "local n = 0 for i = 1, 10 do n = n + i if i == 3 then break end end return n"
        ) => [6.0],
        numeric_for_returns_in_if(
            "local n = 0 for i = 1, 10 do n = n + i if i == 3 then return 'ok' end end return n"
        ) => ["ok"],
    );
}
