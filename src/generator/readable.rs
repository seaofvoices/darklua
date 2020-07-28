use crate::generator::{utils::*, LuaGenerator};
use crate::nodes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatementType {
    Assign,
    Do,
    Call,
    Function,
    GenericFor,
    If,
    LocalAssign,
    LocalFunction,
    NumericFor,
    Repeat,
    While,
    Return,
    Break,
}

impl From<&nodes::Statement> for StatementType {
    fn from(statement: &nodes::Statement) -> Self {
        use nodes::Statement::*;
        match statement {
            Assign(_) => Self::Assign,
            Do(_) => Self::Do,
            Call(_) => Self::Call,
            Function(_) => Self::Function,
            GenericFor(_) => Self::GenericFor,
            If(_) => Self::If,
            LocalAssign(_) => Self::LocalAssign,
            LocalFunction(_) => Self::LocalFunction,
            NumericFor(_) => Self::NumericFor,
            Repeat(_) => Self::Repeat,
            While(_) => Self::While,
        }
    }
}

impl From<&nodes::LastStatement> for StatementType {
    fn from(statement: &nodes::LastStatement) -> Self {
        use nodes::LastStatement::*;
        match statement {
            Break => Self::Break,
            Return(_) => Self::Return,
        }
    }
}

/// This implementation of [LuaGenerator](trait.LuaGenerator.html) attempts to produce Lua code as
/// readable as possible.
#[derive(Debug, Clone)]
pub struct ReadableLuaGenerator {
    column_span: usize,
    indentation: usize,
    current_line_length: usize,
    current_indentation: usize,
    output: String,
    last_push_length: usize,
    can_add_new_line_stack: Vec<bool>,
}

impl ReadableLuaGenerator {
    pub fn new(column_span: usize) -> Self {
        Self {
            column_span,
            indentation: 4,
            current_line_length: 0,
            current_indentation: 0,
            output: String::new(),
            last_push_length: 0,
            can_add_new_line_stack: Vec::new(),
        }
    }

    #[inline]
    fn can_add_new_line(&self) -> bool {
        self.can_add_new_line_stack.last()
            .copied()
            .unwrap_or(true)
    }

    #[inline]
    fn push_can_add_new_line(&mut self, value: bool) {
        self.can_add_new_line_stack.push(value);
    }

    #[inline]
    fn pop_can_add_new_line(&mut self) {
        self.can_add_new_line_stack.pop();
    }

    #[inline]
    fn push_indentation(&mut self) {
        self.current_indentation += 1;
    }

    #[inline]
    fn pop_indentation(&mut self) {
        self.current_indentation -= 1;
    }

    #[inline]
    fn write_indentation(&mut self) {
        let indentation = " ".repeat(self.indentation * self.current_indentation);
        self.raw_push_str(&indentation);
    }

    #[inline]
    fn push_new_line(&mut self) {
        self.output.push('\n');
        self.current_line_length = 0;
    }

    #[inline]
    fn fits_on_current_line(&self, length: usize) -> bool {
        self.current_line_length + length <= self.column_span
    }

    #[inline]
    fn push_space(&mut self) {
        self.output.push(' ');
        self.current_line_length += 1;
    }

    /// Appends a string to the current content of the LuaGenerator. A space may be added
    /// depending of the last character of the current content and the first character pushed.
    fn push_str(&mut self, content: &str) {
        if let Some(next_char) = content.chars().next() {
            self.push_space_if_needed(next_char, content.len());
            self.raw_push_str(content);
        }
    }

    /// Same as the `push_str` function, but for a single character.
    fn push_char(&mut self, character: char) {
        self.push_space_if_needed(character, 1);

        self.output.push(character);
        self.current_line_length += 1;
        self.last_push_length = 1;
    }

    #[inline]
    fn raw_push_str(&mut self, content: &str) {
        self.output.push_str(content);
        self.last_push_length = content.len();
        self.current_line_length += self.last_push_length;
    }

    #[inline]
    fn raw_push_char(&mut self, character: char) {
        self.output.push(character);
        self.last_push_length = 1;
        self.current_line_length += 1;
    }

    #[inline]
    fn needs_space(&self, next_character: char) -> bool {
        is_relevant_for_spacing(&next_character)
        && self.output.chars().last().filter(is_relevant_for_spacing).is_some()
    }

    #[inline]
    fn indent_and_write_block(&mut self, block: &nodes::Block) {
        self.push_indentation();
        self.write_block(block);
        self.pop_indentation();
    }

    fn push_space_if_needed(&mut self, next_character: char, pushed_length: usize) {
        if self.current_line_length == 0 && self.current_indentation != 0 {
            self.write_indentation();
        }

        if self.can_add_new_line() {
            if self.current_line_length >= self.column_span {
                self.push_new_line();
            } else {
                let total_length = self.current_line_length + pushed_length;

                if self.needs_space(next_character) {
                    if total_length + 1 > self.column_span {
                        self.push_new_line();
                    } else {
                        self.push_space();
                    }
                } else {
                    if total_length > self.column_span {
                        self.push_new_line();
                    }
                }
            }
        } else {
            if self.needs_space(next_character) {
                self.push_space();
            }
        }
    }

    /// This function only insert a space or a new line if the given predicate returns true. In
    /// the other case, the string is added to the current generator content.
    fn push_str_and_break_if<F>(&mut self, content: &str, predicate: F)
        where F: Fn(&str) -> bool
    {
        if predicate(self.get_last_push_str()) {
            if self.fits_on_current_line(1 + content.len()) {
                self.push_space();
            } else {
                self.push_new_line();
            }
        } else {
            if !self.fits_on_current_line(content.len()) {
                self.push_new_line();
            }
        }
        self.raw_push_str(content);
    }

    fn get_last_push_str(&self) -> &str {
        self.output.get((self.output.len() - self.last_push_length)..)
            .unwrap_or("")
    }

    fn table_fits_on_line(&self, entries: &Vec<nodes::TableEntry>, _width: usize) -> bool {
        use nodes::TableEntry;

        // small list of simple expressions
        entries.len() < 4
        && entries.iter()
            .all(|entry| match entry {
                TableEntry::Value(value) => self.is_small_expression(value),
                _ => false,
            })
        ||
        entries.len() == 1
        && entries.iter()
            .all(|entry| match entry {
                TableEntry::Field(_identifier, value) => {
                    self.is_small_expression(value)
                }
                TableEntry::Index(key, value) => {
                    self.is_small_expression(key) && self.is_small_expression(value)
                }
                _ => false,
            })
    }

    fn is_small_expression(&self, expression: &nodes::Expression) -> bool {
        use nodes::Expression::*;
        match expression {
            True
            | False
            | Nil
            | Identifier(_)
            | VariableArguments
            | Number(_) => true,
            Table(table) => table.len() == 0,
            _ => false,
        }
    }

    fn write_function_parameters(&mut self, parameters: &Vec<String>, is_variadic: bool) {
        let mut parameters_length = parameters.iter()
            .fold(0, |acc, parameter| acc + parameter.len());
        // add a comma and a space between each parameter
        parameters_length += parameters.len() * 2;

        if is_variadic {
            // add the variadic argument symbol `...`
            parameters_length += 3;
            if parameters.len() > 1 {
                // add comma and space if needed
                parameters_length += 2;
            }
        }

        let last_index = parameters.len().checked_sub(1).unwrap_or(0);

        if self.fits_on_current_line(parameters_length) {
            parameters.iter()
                .enumerate()
                .for_each(|(index, variable)| {
                    self.raw_push_str(variable);

                    if index != last_index {
                        self.raw_push_char(',');
                        self.raw_push_char(' ');
                    }
                });

            if is_variadic {
                if parameters.len() > 0 {
                    self.raw_push_char(',');
                    self.raw_push_char(' ');
                };
                self.raw_push_str("...");
            };
        } else {
            self.push_indentation();

            parameters.iter()
                .enumerate()
                .for_each(|(index, variable)| {
                    self.push_new_line();
                    self.write_indentation();
                    self.raw_push_str(variable);

                    if index != last_index {
                        self.raw_push_char(',');
                    }
                });

            if is_variadic {
                if parameters.len() > 0 {
                    self.raw_push_char(',');
                };
                self.push_new_line();
                self.write_indentation();
                self.raw_push_str("...");
            };

            self.pop_indentation();
            self.push_new_line();
            self.write_indentation();
        }
    }
}

impl Default for ReadableLuaGenerator {
    fn default() -> Self {
        Self::new(80)
    }
}

impl LuaGenerator for ReadableLuaGenerator {
    fn into_string(self) -> String {
        self.output
    }

    fn write_block(&mut self, block: &nodes::Block) {
        let statements = block.get_statements();
        let mut statements = statements.iter().peekable();

        while let Some(statement) = statements.next() {
            let current_type: StatementType = statement.into();

            self.push_can_add_new_line(false);
            self.write_statement(statement);

            if let Some(next_statement) = statements.peek() {
                if starts_with_parenthese(next_statement)
                    && ends_with_prefix(statement)
                {
                    self.push_char(';');
                }

                if current_type != (*next_statement).into() {
                    self.push_new_line();
                }
            }

            self.pop_can_add_new_line();
            self.push_new_line();
        };

        if let Some(last_statement) = block.get_last_statement() {
            if block.get_statements().len() != 0 {
                self.push_new_line();
            }
            self.write_last_statement(last_statement);
            self.push_new_line();
        }
    }

    fn write_last_statement(&mut self, statement: &nodes::LastStatement) {
        use nodes::LastStatement::*;

        match statement {
            Break => self.push_str("break"),
            Return(expressions) => {
                self.push_str("return");
                self.push_can_add_new_line(false);
                let last_index = expressions.len().checked_sub(1).unwrap_or(0);

                expressions.iter()
                    .enumerate()
                    .for_each(|(index, expression)| {
                        self.write_expression(expression);

                        if index != last_index {
                            self.raw_push_char(',');
                            self.raw_push_char(' ');
                        }
                    });

                self.pop_can_add_new_line();
            },
        }
    }

    fn write_assign_statement(&mut self, assign: &nodes::AssignStatement) {
        self.push_can_add_new_line(false);

        let variables = assign.get_variables();
        let last_variable_index = variables.len() - 1;
        variables.iter()
            .enumerate()
            .for_each(|(index, variable)| {
                use nodes::Variable::*;

                match variable {
                    Identifier(identifier) => self.push_str(identifier),
                    Field(field) => self.write_field(field),
                    Index(index) => self.write_index(index),
                }

                if index != last_variable_index {
                    self.raw_push_char(',');
                    self.raw_push_char(' ');
                }
            });

        self.raw_push_str(" = ");

        let values = assign.get_values();
        let last_value_index = values.len() - 1;
        values.iter()
            .enumerate()
            .for_each(|(index, value)| {
                self.write_expression(value);

                if index != last_value_index {
                    self.raw_push_char(',');
                    self.raw_push_char(' ');
                }
            });

        self.pop_can_add_new_line();
    }

    fn write_local_assign(&mut self, assign: &nodes::LocalAssignStatement) {
        self.push_str("local ");

        self.push_can_add_new_line(false);

        let variables = assign.get_variables();
        let last_variable_index = variables.len().checked_sub(1).unwrap_or(0);

        variables.iter()
            .enumerate()
            .for_each(|(index, variable)| {
                self.raw_push_str(variable);

                if index != last_variable_index {
                    self.raw_push_char(',');
                    self.raw_push_char(' ');
                }
            });

        let values = assign.get_values();

        if values.len() > 0 {
            self.raw_push_str(" = ");

            let last_value_index = values.len() - 1;

            values.iter()
                .enumerate()
                .for_each(|(index, value)| {
                    self.write_expression(value);

                    if index != last_value_index {
                        self.raw_push_char(',');
                        self.raw_push_char(' ');
                    }
                });
        };

        self.pop_can_add_new_line();
    }

    fn write_local_function(&mut self, function: &nodes::LocalFunctionStatement) {
        self.push_str("local function ");
        self.raw_push_str(function.get_name());
        self.raw_push_char('(');

        let parameters = function.get_parameters();
        self.write_function_parameters(parameters, function.is_variadic());
        self.raw_push_char(')');

        let block = function.get_block();

        if block.is_empty() {
            self.raw_push_str(" end");
        } else {
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_generic_for(&mut self, generic_for: &nodes::GenericForStatement) {
        self.push_str("for ");

        let identifiers = generic_for.get_identifiers();
        let last_identifier_index = identifiers.len().checked_sub(1).unwrap_or(0);
        identifiers.iter().enumerate()
            .for_each(|(index, identifier)| {
                self.raw_push_str(identifier);

                if index != last_identifier_index {
                    self.raw_push_char(',');
                    self.raw_push_char(' ');
                }
            });

        self.raw_push_str(" in ");

        let expressions = generic_for.get_expressions();
        let last_expression_index = expressions.len().checked_sub(1).unwrap_or(0);
        expressions.iter().enumerate()
            .for_each(|(index, expression)| {
                self.write_expression(expression);

                if index != last_expression_index {
                    self.raw_push_char(',');
                    self.raw_push_char(' ');
                }
            });

        let block = generic_for.get_block();

        if block.is_empty() {
            self.raw_push_str(" do end");
        } else {
            self.push_str("do");
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_numeric_for(&mut self, numeric_for: &nodes::NumericForStatement) {
        self.push_str("for ");

        self.raw_push_str(numeric_for.get_identifier());
        self.raw_push_char('=');
        self.write_expression(numeric_for.get_start());
        self.raw_push_char(',');
        self.raw_push_char(' ');
        self.write_expression(numeric_for.get_end());

        if let Some(step) = numeric_for.get_step() {
            self.raw_push_char(',');
            self.raw_push_char(' ');
            self.write_expression(step);
        }

        let block = numeric_for.get_block();

        if block.is_empty() {
            self.raw_push_str(" do end");
        } else {
            self.push_str("do");
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_if_statement(&mut self, if_statement: &nodes::IfStatement) {
        let branches = if_statement.get_branches();

        branches.iter()
            .enumerate()
            .for_each(|(index, branch)| {
                if index == 0 {
                    self.push_str("if ");
                } else {
                    self.push_str("elseif ");
                }

                self.write_expression(branch.get_condition());
                self.raw_push_str(" then");
                self.push_new_line();
                self.indent_and_write_block(branch.get_block());
            });

        if let Some(else_block) = if_statement.get_else_block() {
            self.push_str("else");
            self.push_new_line();
            self.indent_and_write_block(else_block);
        }

        self.push_str("end");
    }

    fn write_function_statement(&mut self, function: &nodes::FunctionStatement) {
        self.push_str("function ");
        let name = function.get_name();

        self.raw_push_str(name.get_name());
        name.get_field_names().iter()
            .for_each(|field| {
                self.raw_push_char('.');
                self.raw_push_str(field);
            });

        if let Some(method) = name.get_method() {
            self.raw_push_char(':');
            self.raw_push_str(method);
        }

        self.raw_push_char('(');
        self.write_function_parameters(function.get_parameters(), function.is_variadic());
        self.raw_push_char(')');

        let block = function.get_block();

        if block.is_empty() {
            self.raw_push_str(" end");
        } else {
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_do_statement(&mut self, do_statement: &nodes::DoStatement) {
        let block = do_statement.get_block();

        if block.is_empty() {
            self.push_str("do end");
        } else {
            self.push_str("do");
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_repeat_statement(&mut self, repeat: &nodes::RepeatStatement) {
        self.push_str("repeat");

        let block = repeat.get_block();

        if block.is_empty() {
            self.raw_push_str(" until ");
        } else {
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("until ");
        }

        self.write_expression(repeat.get_condition());
    }

    fn write_while_statement(&mut self, while_statement: &nodes::WhileStatement) {
        self.push_str("while");
        self.push_can_add_new_line(false);
        self.write_expression(while_statement.get_condition());
        self.pop_can_add_new_line();

        let block = while_statement.get_block();

        if block.is_empty() {
            self.raw_push_str(" do end");
        } else {
            self.raw_push_str(" do");
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_expression(&mut self, expression: &nodes::Expression) {
        use nodes::Expression::*;
        match expression {
            Binary(binary) => self.write_binary_expression(binary),
            Call(call) => self.write_function_call(call),
            False => self.push_str("false"),
            Field(field) => self.write_field(field),
            Function(function) => self.write_function(function),
            Identifier(identifier) => self.push_str(identifier),
            Index(index) => self.write_index(index),
            Nil => self.push_str("nil"),
            Number(number) => self.write_number(number),
            Parenthese(expression) => {
                self.push_char('(');
                self.push_can_add_new_line(false);
                self.write_expression(expression);
                self.pop_can_add_new_line();
                self.push_char(')');
            }
            String(string) => self.write_string(string),
            Table(table) => self.write_table(table),
            True => self.push_str("true"),
            Unary(unary) => self.write_unary_expression(unary),
            VariableArguments => {
                self.push_str_and_break_if("...", break_variable_arguments);
            }
        }
    }

    fn write_binary_expression(&mut self, binary: &nodes::BinaryExpression) {
        let operator = binary.operator();
        let left = binary.left();
        let right = binary.right();

        if operator.left_needs_parentheses(&left) {
            self.push_char('(');
            self.write_expression(left);
            self.push_char(')');
        } else {
            self.write_expression(left);
        }

        self.push_space();
        self.push_str(binary.operator().to_str());
        self.push_space();

        if operator.right_needs_parentheses(&right) {
            self.push_char('(');
            self.write_expression(right);
            self.push_char(')');
        } else {
            self.write_expression(right);
        }
    }

    fn write_unary_expression(&mut self, unary: &nodes::UnaryExpression) {
        use nodes::{Expression, UnaryOperator::*};

        match unary.operator() {
            Length => self.push_char('#'),
            Minus => self.push_str_and_break_if("-", break_minus),
            Not => self.push_str("not "),
        }

        let expression = unary.get_expression();

        match expression {
            Expression::Binary(binary) if !binary.operator().precedes_unary_expression() => {
                self.push_char('(');
                self.write_expression(expression);
                self.push_char(')');
            },
            _ => self.write_expression(expression),
        }
    }

    fn write_function(&mut self, function: &nodes::FunctionExpression) {
        self.push_str("function(");

        let parameters = function.get_parameters();
        self.write_function_parameters(parameters, function.is_variadic());
        self.raw_push_char(')');

        let block = function.get_block();

        if block.is_empty() {
            self.raw_push_str(" end");
        } else {
            self.push_new_line();
            self.indent_and_write_block(block);
            self.push_str("end");
        }
    }

    fn write_function_call(&mut self, call: &nodes::FunctionCall) {
        self.push_can_add_new_line(false);
        self.write_prefix(call.get_prefix());

        if let Some(method) = &call.get_method() {
            self.push_char(':');
            self.push_str(&method);
        }

        self.write_arguments(call.get_arguments());

        self.pop_can_add_new_line();
    }

    fn write_arguments(&mut self, arguments: &nodes::Arguments) {
        use nodes::Arguments::*;
        match arguments {
            String(string) => self.write_string(string),
            Table(table) => self.write_table(table),
            Tuple(expressions) => {
                self.raw_push_char('(');

                let last_index = expressions.len().checked_sub(1).unwrap_or(0);
                expressions.iter().enumerate()
                    .for_each(|(index, expression)| {
                        self.write_expression(expression);

                        if index != last_index {
                            self.raw_push_char(',');
                            self.raw_push_char(' ');
                        }
                    });

                self.push_char(')');
            }
        }
    }

    fn write_field(&mut self, field: &nodes::FieldExpression) {
        self.push_can_add_new_line(false);
        self.write_prefix(field.get_prefix());
        self.pop_can_add_new_line();

        self.push_char('.');
        self.raw_push_str(&field.get_field());
    }

    fn write_index(&mut self, index: &nodes::IndexExpression) {
        self.push_can_add_new_line(false);

        self.write_prefix(index.get_prefix());

        self.push_char('[');
        self.write_expression(index.get_index());
        self.push_char(']');

        self.pop_can_add_new_line();
    }

    fn write_prefix(&mut self, prefix: &nodes::Prefix) {
        use nodes::Prefix::*;

        match prefix {
            Call(call) => self.write_function_call(call),
            Field(field) => self.write_field(field),
            Identifier(identifier) => self.push_str(identifier),
            Index(index) => self.write_index(index),
            Parenthese(expression) => {
                self.push_char('(');
                self.push_can_add_new_line(false);

                self.write_expression(expression);

                self.pop_can_add_new_line();
                self.push_char(')');
            }
        }
    }

    fn write_table(&mut self, table: &nodes::TableExpression) {
        self.push_char('{');

        let entries = table.get_entries();
        let table_len = entries.len();

        if table_len == 0 {
            self.raw_push_char('}');
        } else {
            let column_space = self.column_span.checked_sub(self.current_line_length)
                .unwrap_or(0);
            if self.table_fits_on_line(entries, column_space) {

                let last_index = table_len.checked_sub(1).unwrap_or(0);

                entries.iter()
                    .enumerate()
                    .for_each(|(index, entry)| {
                        self.write_table_entry(entry);

                        if index != last_index {
                            self.raw_push_char(',');
                            self.raw_push_char(' ');
                        }
                    });

            } else {
                self.push_indentation();

                entries.iter()
                    .for_each(|entry| {
                        self.push_new_line();
                        self.write_indentation();
                        self.write_table_entry(entry);

                        self.raw_push_char(',');
                    });

                self.pop_indentation();
                self.push_new_line();
            }

            self.push_char('}');
        }
    }

    fn write_table_entry(&mut self, entry: &nodes::TableEntry) {
        use nodes::TableEntry::*;

        match entry {
            Field(identifier, value) => {
                self.raw_push_str(identifier);
                self.raw_push_str(" = ");
                self.write_expression(value);
            }
            Index(key, value) => {
                self.raw_push_char('[');
                self.push_can_add_new_line(false);
                self.write_expression(key);
                self.pop_can_add_new_line();
                self.raw_push_str("] = ");
                self.write_expression(value);
            }
            Value(expression) => self.write_expression(expression),
        }
    }

    fn write_number(&mut self, number: &nodes::NumberExpression) {
        use nodes::NumberExpression::*;

        match number {
            Decimal(number) => {
                let float = number.get_raw_float();
                if float.is_nan() {
                    self.push_str("(0/0)");
                } else if float.is_infinite() {
                    self.push_char('(');
                    if float.is_sign_negative() {
                        self.push_char('-');
                    }
                    self.push_str("1/0)")
                } else {
                    self.push_str(&format!("{:.}", float));

                    if let Some(exponent) = number.get_exponent() {
                        let exponent_char = number.is_uppercase()
                            .map(|is_uppercase| if is_uppercase { 'E' } else { 'e' })
                            .unwrap_or('e');
                        self.raw_push_char(exponent_char);
                        self.raw_push_str(&format!("{}", exponent));
                    };
                }
            }
            Hex(number) => {
                self.push_str(&format!(
                    "0{}{:x}",
                    if number.is_x_uppercase() { 'X' } else { 'x' },
                    number.get_raw_integer()
                ));

                if let Some(exponent) = number.get_exponent() {
                    let exponent_char = number.is_exponent_uppercase()
                        .map(|is_uppercase| if is_uppercase { 'P' } else { 'p' })
                        .unwrap_or('p');
                    self.raw_push_char(exponent_char);
                    self.raw_push_str(&format!("{}", exponent));
                };
            }
        }
    }

    fn write_string(&mut self, string: &nodes::StringExpression) {
        let value = string.get_value();
        if string.is_multiline() {
            let mut i = 0;
            let mut equals = "=".repeat(i);

            loop {
                if value.find(&format!("]{}]", equals)).is_none() {
                    break
                } else {
                    i += 1;
                    equals = "=".repeat(i);
                };
            }

            self.push_str_and_break_if(
                &format!("[{}[{}]{}]", equals, value, equals),
                break_long_string
            );

        } else {
            let string = if string.has_single_quote() {
                if string.has_double_quote() {
                    let mut total_escaped = 0;
                    let mut escaped_string = value.to_owned();

                    let mut chars = value.char_indices();

                    while let Some(unescaped_index) = find_not_escaped_from('\'', &mut chars) {
                        escaped_string.insert(unescaped_index + total_escaped, '\\');
                        total_escaped += 1;
                    }

                    format!("'{}'", escaped_string)
                } else {
                    format!("\"{}\"", value)
                }
            } else {
                format!("'{}'", value)
            };

            self.push_str(&string);
        };
    }
}
