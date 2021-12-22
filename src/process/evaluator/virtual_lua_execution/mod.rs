mod execution_effect;
mod local_variable;
mod state;
mod table_storage;

use std::iter;

use crate::{nodes::*, process::FunctionValue};

use execution_effect::ExecutionEffect;
use state::State;
pub use table_storage::{TableId, TableStorage};

use super::{LuaFunction, LuaValue, TableValue, TupleValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationResult {
    None,
    Return(TupleValue),
    Break,
    Continue,
}

impl EvaluationResult {
    #[inline]
    fn is_none(&self) -> bool {
        *self == Self::None
    }

    fn into_tuple(self) -> TupleValue {
        match self {
            Self::Return(tuple) => tuple,
            Self::None | Self::Break | Self::Continue => TupleValue::empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BinaryEvaluationResult {
    Value(LuaValue),
    Left(LuaValue),
    Right(LuaValue),
}

impl From<LuaValue> for BinaryEvaluationResult {
    fn from(value: LuaValue) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualLuaExecution {
    states: Vec<State>,
    current: usize,
    effects: ExecutionEffect,
    max_loop_iteration: usize,
    table_storage: TableStorage,
    perform_mutations: bool,
}

impl Default for VirtualLuaExecution {
    fn default() -> Self {
        Self {
            states: vec![State::new_root(0)],
            current: 0,
            effects: ExecutionEffect::default(),
            max_loop_iteration: 500,
            table_storage: TableStorage::default(),
            perform_mutations: false,
        }
    }
}

impl VirtualLuaExecution {
    pub fn perform_mutations(mut self) -> Self {
        self.perform_mutations = true;
        self
    }

    pub fn with_global_value<S: Into<String>>(mut self, name: S, value: LuaValue) -> Self {
        self.current_state_mut().insert_local(name, value);
        self
    }

    pub fn evaluate_chunk(&mut self, block: &mut Block) -> TupleValue {
        match self.process_block(block) {
            EvaluationResult::Return(value) => value,
            EvaluationResult::None | EvaluationResult::Break | EvaluationResult::Continue => {
                TupleValue::empty()
            }
        }
    }

    fn fork_state(&mut self) -> usize {
        let parent = self.current;
        let id = self.states.len();
        self.states.push(State::new(id, parent));
        self.current = id;
        parent
    }

    fn get_assignment_values<'a>(
        &mut self,
        variable_count: usize,
        values: impl Iterator<Item = &'a mut Expression>,
    ) -> Vec<LuaValue> {
        let mut computed_values = values
            .map(|expression| self.evaluate_expression(expression))
            .collect::<TupleValue>()
            .flatten()
            .into_iter()
            .collect::<Vec<_>>();

        let value_difference = variable_count.saturating_sub(computed_values.len());
        if value_difference > 0 {
            let repeat_value = if computed_values
                .last()
                .filter(|value| matches!(value, LuaValue::Unknown))
                .is_some()
            {
                LuaValue::Unknown
            } else {
                LuaValue::Nil
            };
            computed_values.extend(iter::repeat(repeat_value).take(value_difference));
        }

        computed_values
    }

    pub fn process(&mut self, statement: &mut Statement) -> EvaluationResult {
        match statement {
            Statement::Assign(assign) => {
                let computed_values =
                    self.get_assignment_values(assign.variables_len(), assign.iter_mut_values());

                for (variable, value) in
                    assign.iter_mut_variables().zip(computed_values.into_iter())
                {
                    self.assign_variable(variable, value);
                }
                EvaluationResult::None
            }
            Statement::Do(do_statement) => self.process_block(do_statement.mutate_block()),
            Statement::Call(call) => {
                self.evaluate_call(call);
                EvaluationResult::None
            }
            Statement::CompoundAssign(assign) => {
                let value = self.evaluate_compound_assign(assign);
                self.assign_variable(assign.mutate_variable(), value);
                EvaluationResult::None
            }
            Statement::Function(function) => {
                let name = function.get_name();
                // TODO: build function name into a field expression and apply the function value

                let current_state = self.current;
                let root_identifier = name.get_name().get_name();
                if let Some(state) = self
                    .find_ancestor_with_identifier(root_identifier)
                    .and_then(|id| self.mut_state(id))
                {
                    state.assign_identifier(
                        root_identifier,
                        LuaFunction::from(&*function)
                            .with_parent_state(current_state)
                            .into(),
                    );
                    self.effects.add(root_identifier);
                }

                // TODO: do not run function yet and blur variables
                // let parent_id = self.fork_state();
                // let function_state_id = self.current_state().id();

                // for parameter in function.iter_parameters() {
                //     self.mut_state(function_state_id)
                //         .expect("function state should exist")
                //         .insert_local(parameter.get_name(), LuaValue::Unknown);
                // }

                // self.process_conditional_block(function.mutate_block());

                // self.current = parent_id;

                EvaluationResult::None
            }
            Statement::GenericFor(for_statement) => {
                // TODO
                self.process_conditional_block(for_statement.mutate_block());
                EvaluationResult::None
            }
            Statement::If(if_statement) => {
                let mut else_should_run = Some(true);
                for branch in if_statement.iter_mut_branches() {
                    match self
                        .evaluate_expression(branch.mutate_condition())
                        .is_truthy()
                    {
                        Some(true) => {
                            if else_should_run.is_some() {
                                let result = self.process_block(branch.mutate_block());
                                if !result.is_none() {
                                    return result;
                                }
                                else_should_run = Some(false);
                            } else {
                                self.process_conditional_block(branch.mutate_block());
                            }
                            break;
                        }
                        Some(false) => continue,
                        None => {
                            // this branch may run, so we need to blur all locals it captures
                            self.process_conditional_block(branch.mutate_block());
                            else_should_run = None;
                        }
                    }
                }

                if let Some(else_block) = if_statement.mutate_else_block() {
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
                let computed_values =
                    self.get_assignment_values(assign.variables_len(), assign.iter_mut_values());

                for (identifier, value) in assign.iter_variables().zip(computed_values.into_iter())
                {
                    self.current_state_mut()
                        .insert_local(identifier.get_name(), value);
                }

                EvaluationResult::None
            }
            Statement::LocalFunction(function) => {
                let name = function.get_name();
                let current_state = self.current;
                self.current_state_mut().insert_local(
                    name,
                    LuaFunction::from(&*function)
                        .with_parent_state(current_state)
                        .into(),
                );

                // TODO: do not blur variables yet
                // let parent_id = self.fork_state();
                // let function_state_id = self.current_state().id();

                // for parameter in function.iter_parameters() {
                //     self.mut_state(function_state_id)
                //         .expect("local function state should exist")
                //         .insert_local(parameter.get_name(), LuaValue::Unknown);
                // }

                // self.process_conditional_block(function.mutate_block());

                // self.current = parent_id;

                EvaluationResult::None
            }
            Statement::NumericFor(for_statement) => {
                let init = self.evaluate_expression(for_statement.mutate_start());
                let end = self.evaluate_expression(for_statement.mutate_end());
                let step = for_statement
                    .mutate_step()
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
                    let variable_name = for_statement.get_identifier().get_name().to_owned();

                    let result = loop {
                        if iteration >= self.max_loop_iteration {
                            self.process_conditional_block(for_statement.mutate_block());
                            break EvaluationResult::None;
                        }
                        if (step > 0.0 && variable > end) || (step < 0.0 && variable < end) {
                            break EvaluationResult::None;
                        }

                        self.mut_state(for_loop_state_id)
                            .expect("for loop state should exist")
                            .insert_local(&variable_name, LuaValue::Number(variable));

                        let result =
                            self.process_block_without_mutations(for_statement.mutate_block());
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
                    self.process_conditional_block(for_statement.mutate_block());
                    EvaluationResult::None
                }
            }
            Statement::Repeat(repeat) => {
                let mut iteration = 0;
                loop {
                    if iteration >= self.max_loop_iteration {
                        self.process_conditional_block(repeat.mutate_block());
                        self.process_conditional_expression(repeat.mutate_condition());
                        break EvaluationResult::None;
                    }
                    let result = self.process_block_without_mutations(repeat.mutate_block());
                    match result {
                        EvaluationResult::Return(_) => return result,
                        EvaluationResult::Break => {
                            break EvaluationResult::None;
                        }
                        EvaluationResult::None | EvaluationResult::Continue => {}
                    }
                    match self
                        .evaluate_expression(repeat.mutate_condition())
                        .is_truthy()
                    {
                        Some(false) => {}
                        Some(true) => break EvaluationResult::None,
                        None => {
                            self.process_conditional_block(repeat.mutate_block());
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
                        self.process_conditional_expression(while_statement.mutate_condition());
                        self.process_conditional_block(while_statement.mutate_block());
                        break EvaluationResult::None;
                    }
                    match self
                        .evaluate_expression(while_statement.mutate_condition())
                        .is_truthy()
                    {
                        Some(true) => {
                            let result = self
                                .process_block_without_mutations(while_statement.mutate_block());
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
                            self.process_conditional_block(while_statement.mutate_block());
                            break EvaluationResult::None;
                        }
                    }
                    iteration += 1;
                }
            }
        }
    }

    fn assign_variable(&mut self, variable: &mut Variable, value: LuaValue) {
        match variable {
            Variable::Identifier(identifier) => self.assign_identifier(identifier, value),
            Variable::Field(field) => {
                let key = LuaValue::from(field.get_field().get_name().as_str());
                self.assign_prefix(field.mutate_prefix(), key, value)
            }
            Variable::Index(index) => {
                let key = self.evaluate_expression(index.mutate_index());
                self.assign_prefix(index.mutate_prefix(), key, value);
            }
        }
    }

    fn assign_identifier(&mut self, identifier: &Identifier, value: LuaValue) {
        let name = identifier.get_name();
        if let Some(state) = self
            .find_ancestor_with_identifier(name)
            .and_then(|id| self.mut_state(id))
        {
            state.assign_identifier(name, value);
            self.effects.add(name);
        }
    }

    fn assign_prefix(&mut self, prefix: &mut Prefix, key: LuaValue, value: LuaValue) {
        match self.evaluate_prefix(prefix) {
            LuaValue::TableRef(table_id) => {
                if let Some(table) = self.table_storage.mutate(table_id) {
                    table.insert(key, value);
                }
            }
            _ => {}
        };
    }

    fn process_block(&mut self, block: &mut Block) -> EvaluationResult {
        let parent_id = self.fork_state();

        for statement in block.iter_mut_statements() {
            let result = self.process(statement);

            match result {
                EvaluationResult::Return(_)
                | EvaluationResult::Break
                | EvaluationResult::Continue => return result,
                EvaluationResult::None => {}
            }
        }

        let result = if let Some(last) = block.mutate_last_statement() {
            self.process_last_statement(last)
        } else {
            EvaluationResult::None
        };

        self.current = parent_id;
        result
    }

    fn process_last_statement(&mut self, statement: &mut LastStatement) -> EvaluationResult {
        match statement {
            LastStatement::Break(_) => EvaluationResult::Break,
            LastStatement::Continue(_) => EvaluationResult::Continue,
            LastStatement::Return(statement) => EvaluationResult::Return(
                statement
                    .iter_mut_expressions()
                    .map(|expression| self.evaluate_expression(expression))
                    .collect::<TupleValue>()
                    .flatten(),
            ),
        }
    }

    fn process_block_without_mutations(&mut self, block: &mut Block) -> EvaluationResult {
        let restore_mutations = self.perform_mutations;
        self.perform_mutations = false;
        let result = self.process_block(block);
        self.perform_mutations = restore_mutations;
        result
    }

    fn process_expression_without_mutations(&mut self, expression: &mut Expression) -> LuaValue {
        let restore_mutations = self.perform_mutations;
        self.perform_mutations = false;
        let result = self.evaluate_expression(expression);
        self.perform_mutations = restore_mutations;
        result
    }

    fn process_conditional_expression(&mut self, expression: &mut Expression) {
        self.effects.enable();
        self.process_expression_without_mutations(expression);
        self.disable_effects();
    }

    fn process_conditional_block(&mut self, block: &mut Block) {
        self.effects.enable();
        self.process_block_without_mutations(block);
        self.disable_effects();
    }

    fn disable_effects(&mut self) {
        for identifier in self.effects.disable() {
            if let Some(state) = self
                .find_ancestor_with_identifier(&identifier)
                .and_then(|id| self.mut_state(id))
            {
                state.assign_identifier(&identifier, LuaValue::Unknown);
            }
        }
    }

    fn evaluate_compound_assign(&mut self, assign: &mut CompoundAssignStatement) -> LuaValue {
        // evaluate variable first, then the expression value
        let left = self.evaluate_variable(assign.mutate_variable());
        let right = self.evaluate_expression(assign.mutate_value());
        match assign.get_operator() {
            CompoundOperator::Plus => self.evaluate_math_values(left, right, |a, b| a + b),
            CompoundOperator::Minus => self.evaluate_math_values(left, right, |a, b| a - b),
            CompoundOperator::Asterisk => self.evaluate_math_values(left, right, |a, b| a * b),
            CompoundOperator::Slash => self.evaluate_math_values(left, right, |a, b| a / b),
            CompoundOperator::Caret => self.evaluate_math_values(left, right, |a, b| a.powf(b)),
            CompoundOperator::Percent => {
                self.evaluate_math_values(left, right, |a, b| a - b * (a / b).floor())
            }
            CompoundOperator::Concat => self.evaluate_concat(left, right),
        }
    }

    pub fn evaluate_expression(&mut self, expression: &mut Expression) -> LuaValue {
        match expression {
            Expression::False(_) => LuaValue::False,
            Expression::Function(function) => LuaFunction::from(&*function)
                .with_parent_state(self.current)
                .into(),
            Expression::Nil(_) => LuaValue::Nil,
            Expression::Number(number) => LuaValue::from(number.compute_value()),
            Expression::String(string) => LuaValue::from(string.get_value()),
            Expression::Table(table) => self.evaluate_table(table),
            Expression::True(_) => LuaValue::True,
            Expression::Binary(binary) => match self.evaluate_binary(binary) {
                BinaryEvaluationResult::Value(value) => self.replace_expression(expression, value),
                BinaryEvaluationResult::Left(value) => {
                    if self.perform_mutations {
                        *expression = value
                            .to_expression()
                            .unwrap_or_else(|| binary.left().clone())
                    }
                    value
                }
                BinaryEvaluationResult::Right(value) => {
                    if self.perform_mutations {
                        *expression = value
                            .to_expression()
                            .unwrap_or_else(|| binary.right().clone())
                    }
                    value
                }
            },
            Expression::Unary(unary) => {
                let value = self.evaluate_unary(unary);
                self.replace_expression(expression, value)
            }
            Expression::Parenthese(parenthese) => {
                let value = self.evaluate_parenthese(parenthese);
                self.replace_expression(expression, value)
            }
            Expression::Identifier(identifier) => {
                let value = self.evaluate_identifier(identifier);
                self.replace_expression(expression, value)
            }
            Expression::Field(field) => {
                let value = self.evaluate_field(field);
                self.replace_expression(expression, value)
            }
            Expression::Index(index) => {
                let value = self.evaluate_index(index);
                self.replace_expression(expression, value)
            }
            Expression::Call(call) => {
                let value = self.evaluate_call(call).into();
                self.replace_expression(expression, value)
            }
            Expression::VariableArguments(_) => LuaValue::Unknown,
        }
    }

    fn replace_expression(&self, expression: &mut Expression, value: LuaValue) -> LuaValue {
        if self.perform_mutations {
            if let Some(new_expression) = value.to_expression() {
                *expression = new_expression;
            }
        }
        value
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

    fn find_ancestor_with_identifier(&self, identifier: &str) -> Option<usize> {
        let mut current = self.current_state();
        while !current.has_identifier(identifier) {
            current = self.get_state(current.parent()?)?;
        }
        Some(current.id())
    }

    fn evaluate_call(&mut self, call: &mut FunctionCall) -> TupleValue {
        // TODO: if in conditional mode, there may be more processing to do
        let prefix = self
            .evaluate_prefix(call.mutate_prefix())
            .coerce_to_single_value();
        let arguments = self.evaluate_arguments(call.mutate_arguments());
        match prefix {
            LuaValue::Function(function) => {
                match function {
                    FunctionValue::Lua(mut lua_function) => {
                        let parent_id = self.fork_state();
                        let function_state_id = self.current_state().id();

                        let mut arguments_iter = arguments.into_iter();

                        for parameter in lua_function.iter_parameters() {
                            let parameter_value = arguments_iter.next()
                                .unwrap_or_else(|| LuaValue::Nil);

                            self.mut_state(function_state_id)
                                .expect("function state should exist")
                                .insert_local(parameter, parameter_value);
                        }

                        // TODO: put remaining arguments into `...`
                        let result = self.process_block_without_mutations(lua_function.mutate_block());

                        self.current = parent_id;

                        result.into_tuple()
                    },
                    FunctionValue::Engine(engine) => {
                        engine.execute(arguments)
                    },
                    FunctionValue::Unknown => LuaValue::Unknown.into(),
                }
            }
            LuaValue::Nil
            | LuaValue::Table(_) // TODO: table can be called
            | LuaValue::TableRef(_)
            | LuaValue::Number(_)
            | LuaValue::String(_)
            | LuaValue::True
            | LuaValue::False
            | LuaValue::Unknown => {
                self.pass_argument_to_unknown_function(arguments);
                LuaValue::Unknown.into()
            }
            // unreachable because of the call to `coerce_to_single_value`
            LuaValue::Tuple(_) => LuaValue::Unknown.into(),
        }
    }

    fn pass_argument_to_unknown_function(&mut self, arguments: TupleValue) {
        for value in arguments.iter() {
            if let LuaValue::TableRef(id) = &value {
                if let Some(table) = self.table_storage.mutate(*id) {
                    table.clear();
                    table.set_unknown_mutations();
                }
            }
        }
    }

    fn evaluate_arguments(&mut self, arguments: &mut Arguments) -> TupleValue {
        match arguments {
            Arguments::Tuple(tuple) => tuple
                .iter_mut_values()
                .map(|value| self.evaluate_expression(value))
                .collect::<TupleValue>()
                .flatten(),
            Arguments::String(string) => TupleValue::singleton(string.get_value()),
            Arguments::Table(table) => TupleValue::singleton(self.evaluate_table(table)),
        }
    }

    fn evaluate_binary(&mut self, binary: &mut BinaryExpression) -> BinaryEvaluationResult {
        match binary.operator() {
            BinaryOperator::And => {
                let left = self.evaluate_expression(binary.mutate_left());
                match left.is_truthy() {
                    Some(true) => BinaryEvaluationResult::Right(
                        self.evaluate_expression(binary.mutate_right()),
                    ),
                    Some(false) => BinaryEvaluationResult::Left(left),
                    None => {
                        self.evaluate_expression(binary.mutate_right());
                        BinaryEvaluationResult::Value(LuaValue::Unknown)
                    }
                }
            }
            BinaryOperator::Or => {
                let left = self.evaluate_expression(binary.mutate_left());
                match left.is_truthy() {
                    Some(true) => BinaryEvaluationResult::Left(left),
                    Some(false) => BinaryEvaluationResult::Right(
                        self.evaluate_expression(binary.mutate_right()),
                    ),
                    None => {
                        self.evaluate_expression(binary.mutate_right());
                        BinaryEvaluationResult::Value(LuaValue::Unknown)
                    }
                }
            }
            BinaryOperator::Equal => {
                let left = self.evaluate_expression(binary.mutate_left());
                let right = self.evaluate_expression(binary.mutate_right());
                self.evaluate_equal(&left, &right).into()
            }
            BinaryOperator::NotEqual => {
                let left = self.evaluate_expression(binary.mutate_left());
                let right = self.evaluate_expression(binary.mutate_right());
                let result = self.evaluate_equal(&left, &right);

                match result {
                    LuaValue::True => LuaValue::False,
                    LuaValue::False => LuaValue::True,
                    _ => LuaValue::Unknown,
                }
                .into()
            }
            BinaryOperator::Plus => self.evaluate_math(binary, |a, b| a + b).into(),
            BinaryOperator::Minus => self.evaluate_math(binary, |a, b| a - b).into(),
            BinaryOperator::Asterisk => self.evaluate_math(binary, |a, b| a * b).into(),
            BinaryOperator::Slash => self.evaluate_math(binary, |a, b| a / b).into(),
            BinaryOperator::Caret => self.evaluate_math(binary, |a, b| a.powf(b)).into(),
            BinaryOperator::Percent => self
                .evaluate_math(binary, |a, b| a - b * (a / b).floor())
                .into(),
            BinaryOperator::Concat => {
                let left = self.evaluate_expression(binary.mutate_left());
                let right = self.evaluate_expression(binary.mutate_right());

                self.evaluate_concat(left, right).into()
            }
            BinaryOperator::LowerThan => self.evaluate_relational(binary, |a, b| a < b).into(),
            BinaryOperator::LowerOrEqualThan => {
                self.evaluate_relational(binary, |a, b| a <= b).into()
            }
            BinaryOperator::GreaterThan => self.evaluate_relational(binary, |a, b| a > b).into(),
            BinaryOperator::GreaterOrEqualThan => {
                self.evaluate_relational(binary, |a, b| a >= b).into()
            }
        }
    }

    fn evaluate_concat(&mut self, left: LuaValue, right: LuaValue) -> LuaValue {
        match (left.string_coercion(), right.string_coercion()) {
            (LuaValue::String(mut left), LuaValue::String(right)) => {
                left.push_str(&right);
                LuaValue::String(left)
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
            (LuaValue::Number(a), LuaValue::Number(b)) => {
                LuaValue::from((a - b).abs() < f64::EPSILON)
            }
            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::from(a == b),
            _ => LuaValue::False,
        }
    }

    fn evaluate_math<F>(&mut self, binary: &mut BinaryExpression, operation: F) -> LuaValue
    where
        F: Fn(f64, f64) -> f64,
    {
        let left = self
            .evaluate_expression(binary.mutate_left())
            .number_coercion();
        let right = self
            .evaluate_expression(binary.mutate_right())
            .number_coercion();

        if let LuaValue::Number(left) = left {
            if let LuaValue::Number(right) = right {
                LuaValue::Number(operation(left, right))
            } else {
                LuaValue::Unknown
            }
        } else {
            LuaValue::Unknown
        }
    }

    fn evaluate_math_values<F>(&mut self, left: LuaValue, right: LuaValue, operation: F) -> LuaValue
    where
        F: Fn(f64, f64) -> f64,
    {
        let left = left.number_coercion();

        if let LuaValue::Number(left) = left {
            let right = right.number_coercion();

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
        &mut self,
        expression: &mut BinaryExpression,
        operation: F,
    ) -> LuaValue
    where
        F: Fn(f64, f64) -> bool,
    {
        let left = self.evaluate_expression(expression.mutate_left());
        let right = self.evaluate_expression(expression.mutate_right());

        match (left, right) {
            (LuaValue::Number(left), LuaValue::Number(right)) => {
                if operation(left, right) {
                    LuaValue::True
                } else {
                    LuaValue::False
                }
            }
            (LuaValue::String(left), LuaValue::String(right)) => {
                self.compare_strings(&left, &right, expression.operator())
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

    fn evaluate_unary(&mut self, unary: &mut UnaryExpression) -> LuaValue {
        let inner = self.evaluate_expression(unary.mutate_expression());
        match unary.operator() {
            UnaryOperator::Not => inner
                .is_truthy()
                .map(|value| LuaValue::from(!value))
                .unwrap_or(LuaValue::Unknown),
            UnaryOperator::Minus => match inner.number_coercion() {
                LuaValue::Number(value) => LuaValue::from(-value),
                _ => LuaValue::Unknown,
            },
            _ => LuaValue::Unknown,
        }
    }

    fn evaluate_table(&mut self, table: &mut TableExpression) -> LuaValue {
        let last_index = table.len().saturating_sub(1);
        let table_value = table.iter_mut_entries().enumerate().fold(
            TableValue::default(),
            |mut table_value, (i, entry)| match entry {
                TableEntry::Field(field) => table_value.with_entry(
                    LuaValue::from(field.get_field().get_name().as_str()),
                    self.evaluate_expression(field.mutate_value()),
                ),
                TableEntry::Index(index) => table_value.with_entry(
                    self.evaluate_expression(index.mutate_key()),
                    self.evaluate_expression(index.mutate_value()),
                ),
                TableEntry::Value(value) => {
                    if last_index == i && matches!(value, Expression::VariableArguments(_)) {
                        match self.evaluate_expression(value) {
                            LuaValue::Tuple(tuple) => {
                                for lua_value in tuple.into_iter() {
                                    table_value.push_element(lua_value);
                                }
                                table_value
                            }
                            lua_value => table_value.with_array_element(lua_value),
                        }
                    } else {
                        table_value.with_array_element(self.evaluate_expression(value))
                    }
                }
            },
        );
        let id = self.table_storage.insert(table_value);
        LuaValue::TableRef(id)
    }

    fn evaluate_parenthese(&mut self, parenthese: &mut ParentheseExpression) -> LuaValue {
        self.evaluate_expression(parenthese.mutate_inner_expression())
            .coerce_to_single_value()
    }

    fn evaluate_identifier(&self, identifier: &Identifier) -> LuaValue {
        self.current_state()
            .read(identifier.get_name(), self)
            .unwrap_or(LuaValue::Unknown)
    }

    fn evaluate_field(&mut self, field: &mut FieldExpression) -> LuaValue {
        match self.evaluate_prefix(field.mutate_prefix()).coerce_to_single_value() {
            LuaValue::Table(table) => {
                let key = field.get_field().get_name().to_owned().into();
                table.get(&key).clone()
            }
            LuaValue::TableRef(id) => {
                self.table_storage.get(id)
                    .expect("table should exist")
                    .get(&field.get_field().get_name().to_owned().into())
                    .clone()
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

    fn evaluate_index(&mut self, index: &mut IndexExpression) -> LuaValue {
        let key = self
            .evaluate_expression(index.mutate_index())
            .coerce_to_single_value();
        match self.evaluate_prefix(index.mutate_prefix()).coerce_to_single_value() {
            LuaValue::Table(table) => {
                table.get(&key).clone()
            }
            LuaValue::TableRef(id) => {
                self.table_storage.get(id)
                    .expect("table should exist")
                    .get(&key).clone()
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

    fn evaluate_prefix(&mut self, prefix: &mut Prefix) -> LuaValue {
        match prefix {
            Prefix::Field(field) => self.evaluate_field(field),
            Prefix::Identifier(identifier) => self.evaluate_identifier(identifier),
            Prefix::Index(index) => self.evaluate_index(index),
            Prefix::Parenthese(parenthese) => self.evaluate_parenthese(parenthese),
            Prefix::Call(call) => self.evaluate_call(call).into(),
        }
    }

    fn evaluate_variable(&mut self, variable: &mut Variable) -> LuaValue {
        match variable {
            Variable::Identifier(identifier) => self.evaluate_identifier(identifier),
            Variable::Field(field) => self.evaluate_field(field),
            Variable::Index(index) => self.evaluate_index(index),
        }
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
                    let mut block = crate::Parser::default()
                        .parse($code)
                        .expect("code should parse");

                    let mut state = VirtualLuaExecution::default();

                    pretty_assertions::assert_eq!(
                        state.evaluate_chunk(&mut block),
                        TupleValue::new(vec![ $( LuaValue::from($result), )* ])
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
        define_table_and_return_field("local a = { b = 'ok' } return a.b") => ["ok"],
    );
}
