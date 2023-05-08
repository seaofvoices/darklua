use std::iter;

use crate::{
    generator::{utils, LuaGenerator},
    nodes::*,
};

/// This implementation of [LuaGenerator](trait.LuaGenerator.html) outputs the
/// AST nodes from the tokens associated with each of them.
#[derive(Debug, Clone)]
pub struct TokenBasedLuaGenerator<'a> {
    original_code: &'a str,
    output: String,
    currently_commenting: bool,
    current_line: usize,
}

impl<'a> TokenBasedLuaGenerator<'a> {
    pub fn new(original_code: &'a str) -> Self {
        Self {
            original_code,
            output: String::new(),
            currently_commenting: false,
            current_line: 1,
        }
    }

    fn push_str(&mut self, string: &str) {
        self.current_line += utils::count_new_lines(string);
        self.output.push_str(string);
    }

    fn write_trivia(&mut self, trivia: &Trivia) {
        let content = trivia.read(self.original_code);
        self.push_str(content);

        match trivia.kind() {
            TriviaKind::Comment => {
                if is_single_line_comment(content) {
                    self.currently_commenting = true;
                }
            }
            TriviaKind::Whitespace => {
                if self.currently_commenting && content.contains('\n') {
                    self.currently_commenting = false
                }
            }
        }
    }

    #[inline]
    fn write_token(&mut self, token: &Token) {
        self.write_token_options(token, true)
    }

    fn write_token_options(&mut self, token: &Token, space_check: bool) {
        for trivia in token.iter_leading_trivia() {
            self.write_trivia(trivia);
        }

        let content = token.read(self.original_code);

        if self.currently_commenting {
            self.uncomment();
        }

        if let Some(line_number) = token.get_line_number() {
            while line_number > self.current_line {
                self.output.push('\n');
                self.current_line += 1;
            }
        }

        if space_check {
            if let Some(next_character) = content.chars().next() {
                if self.needs_space(next_character) {
                    self.output.push(' ');
                }
            }
        }

        self.push_str(content);

        for trivia in token.iter_trailing_trivia() {
            self.write_trivia(trivia);
        }
    }

    fn write_block_with_tokens(&mut self, block: &Block, tokens: &BlockTokens) {
        let mut iterator = block.iter_statements().enumerate().peekable();

        while let Some((index, statement)) = iterator.next() {
            self.write_statement(statement);

            if let Some(semicolon) = tokens.semicolons.get(index).unwrap_or(&None) {
                self.write_token(semicolon);
            } else if let Some((_, next_statement)) = iterator.peek() {
                if utils::starts_with_parenthese(next_statement)
                    && utils::ends_with_prefix(statement)
                {
                    self.write_symbol(";");
                }
            };
        }

        if let Some(statement) = block.get_last_statement() {
            self.write_last_statement(statement);
        }
    }

    fn write_return_with_tokens(&mut self, statement: &ReturnStatement, tokens: &ReturnTokens) {
        self.write_token(&tokens.r#return);

        let last_index = statement.len().saturating_sub(1);
        statement
            .iter_expressions()
            .enumerate()
            .for_each(|(i, expression)| {
                self.write_expression(expression);
                if i < last_index {
                    if let Some(comma) = tokens.commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });
    }

    fn write_assign_with_tokens(&mut self, assign: &AssignStatement, tokens: &AssignTokens) {
        let last_variable_index = assign.variables_len().saturating_sub(1);
        assign
            .iter_variables()
            .enumerate()
            .for_each(|(i, variable)| {
                self.write_variable(variable);
                if i < last_variable_index {
                    if let Some(comma) = tokens.variable_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        self.write_token(&tokens.equal);
        let last_value_index = assign.values_len().saturating_sub(1);
        assign.iter_values().enumerate().for_each(|(i, value)| {
            self.write_expression(value);
            if i < last_value_index {
                if let Some(comma) = tokens.value_commas.get(i) {
                    self.write_token(comma);
                } else {
                    self.write_symbol(",");
                }
            }
        });
    }

    fn write_do_with_tokens(&mut self, do_statement: &DoStatement, tokens: &DoTokens) {
        self.write_token(&tokens.r#do);
        self.write_block(do_statement.get_block());
        self.write_token(&tokens.end);
    }

    fn write_function_call_with_tokens(
        &mut self,
        call: &FunctionCall,
        tokens: &FunctionCallTokens,
    ) {
        self.write_prefix(call.get_prefix());
        if let Some(method) = call.get_method() {
            if let Some(colon) = &tokens.colon {
                self.write_token(colon);
            } else {
                self.write_symbol(":");
            }
            self.write_identifier(method);
        }
        self.write_arguments(call.get_arguments());
    }

    fn write_parenthese_with_tokens(
        &mut self,
        parenthese: &ParentheseExpression,
        tokens: &ParentheseTokens,
    ) {
        self.write_token(&tokens.left_parenthese);
        self.write_expression(parenthese.inner_expression());
        self.write_token(&tokens.right_parenthese);
    }

    fn write_tuple_arguments_with_tokens(
        &mut self,
        arguments: &TupleArguments,
        tokens: &TupleArgumentsTokens,
    ) {
        self.write_token(&tokens.opening_parenthese);

        let last_value_index = arguments.len().saturating_sub(1);
        arguments.iter_values().enumerate().for_each(|(i, value)| {
            self.write_expression(value);
            if i < last_value_index {
                if let Some(comma) = tokens.commas.get(i) {
                    self.write_token(comma);
                } else {
                    self.write_symbol(",");
                }
            }
        });

        self.write_token(&tokens.closing_parenthese);
    }

    fn write_table_with_tokens(&mut self, table: &TableExpression, tokens: &TableTokens) {
        self.write_token(&tokens.opening_brace);

        let last_index = table.len().saturating_sub(1);
        table.iter_entries().enumerate().for_each(|(i, entry)| {
            self.write_table_entry(entry);
            if let Some(separator) = tokens.separators.get(i) {
                self.write_token(separator);
            } else if i < last_index {
                self.write_symbol(",");
            }
        });

        self.write_token(&tokens.closing_brace);
    }

    fn write_table_field_with_tokens(&mut self, entry: &TableFieldEntry, token: &Token) {
        self.write_identifier(entry.get_field());
        self.write_token(token);
        self.write_expression(entry.get_value());
    }

    fn write_table_index_with_tokens(
        &mut self,
        entry: &TableIndexEntry,
        tokens: &TableIndexEntryTokens,
    ) {
        self.write_token(&tokens.opening_bracket);
        self.write_expression(entry.get_key());
        self.write_token(&tokens.closing_bracket);
        self.write_token(&tokens.equal);
        self.write_expression(entry.get_value());
    }

    fn write_field_with_token(&mut self, field: &FieldExpression, token: &Token) {
        self.write_prefix(field.get_prefix());
        self.write_token_options(token, false);
        self.write_identifier(field.get_field());
    }

    fn write_index_with_tokens(&mut self, index: &IndexExpression, tokens: &IndexExpressionTokens) {
        self.write_prefix(index.get_prefix());
        self.write_token(&tokens.opening_bracket);
        self.write_expression(index.get_index());
        self.write_token(&tokens.closing_bracket);
    }

    fn write_if_expression_with_token(
        &mut self,
        if_expression: &IfExpression,
        tokens: &IfExpressionTokens,
    ) {
        self.write_token(&tokens.r#if);
        self.write_expression(if_expression.get_condition());
        self.write_token(&tokens.then);
        self.write_expression(if_expression.get_result());

        for branch in if_expression.iter_branches() {
            if let Some(tokens) = branch.get_tokens() {
                self.write_if_expression_branch_with_tokens(branch, tokens);
            } else {
                self.write_if_expression_branch_with_tokens(
                    branch,
                    &self.generate_if_expression_branch_tokens(branch),
                );
            }
        }

        self.write_token(&tokens.r#else);
        self.write_expression(if_expression.get_else_result());
    }

    fn write_if_expression_branch_with_tokens(
        &mut self,
        branch: &ElseIfExpressionBranch,
        tokens: &ElseIfExpressionBranchTokens,
    ) {
        self.write_token(&tokens.elseif);
        self.write_expression(branch.get_condition());
        self.write_token(&tokens.then);
        self.write_expression(branch.get_result());
    }

    fn write_compound_assign_with_tokens(
        &mut self,
        assign: &CompoundAssignStatement,
        tokens: &CompoundAssignTokens,
    ) {
        self.write_variable(assign.get_variable());
        self.write_token(&tokens.operator);
        self.write_expression(assign.get_value());
    }

    fn write_function_statement_with_tokens(
        &mut self,
        function: &FunctionStatement,
        tokens: &FunctionStatementTokens,
    ) {
        self.write_token(&tokens.function);

        let name = function.get_name();
        if let Some(tokens) = name.get_tokens() {
            self.write_function_name_with_tokens(name, tokens);
        } else {
            self.write_function_name_with_tokens(name, &self.generate_function_name_tokens(name));
        }

        self.write_token(&tokens.opening_parenthese);

        let parameter_count = function.parameters_count();
        let last_parameter_index = parameter_count.saturating_sub(1);
        function
            .iter_parameters()
            .enumerate()
            .for_each(|(i, param)| {
                self.write_identifier(param);
                if i < last_parameter_index {
                    if let Some(comma) = tokens.parameter_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        if function.is_variadic() {
            if function.has_parameters() {
                if let Some(comma) = tokens.parameter_commas.get(last_parameter_index) {
                    self.write_token(comma);
                } else {
                    self.write_symbol(",");
                }
            }

            if let Some(token) = &tokens.variable_arguments {
                self.write_token(token);
            } else {
                self.write_symbol("...");
            }
        }

        self.write_token(&tokens.closing_parenthese);

        self.write_block(function.get_block());

        self.write_token(&tokens.end);
    }

    fn write_function_name_with_tokens(
        &mut self,
        name: &FunctionName,
        tokens: &FunctionNameTokens,
    ) {
        self.write_identifier(name.get_name());

        name.get_field_names()
            .iter()
            .enumerate()
            .for_each(|(i, field)| {
                if let Some(period) = tokens.periods.get(i) {
                    self.write_token(period);
                } else {
                    self.write_symbol(".");
                }
                self.write_identifier(field);
            });

        if let Some(method) = name.get_method() {
            if let Some(colon) = &tokens.colon {
                self.write_token(colon);
            } else {
                self.write_symbol(":");
            }
            self.write_identifier(method);
        }
    }

    fn write_generic_for_with_tokens(
        &mut self,
        generic_for: &GenericForStatement,
        tokens: &GenericForTokens,
    ) {
        self.write_token(&tokens.r#for);

        let last_identifier_index = generic_for.identifiers_len().saturating_sub(1);
        generic_for
            .iter_identifiers()
            .enumerate()
            .for_each(|(i, identifier)| {
                self.write_identifier(identifier);
                if i < last_identifier_index {
                    if let Some(comma) = tokens.identifier_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        self.write_token(&tokens.r#in);

        let last_expression_index = generic_for.expressions_len().saturating_sub(1);
        generic_for
            .iter_expressions()
            .enumerate()
            .for_each(|(i, expression)| {
                self.write_expression(expression);
                if i < last_expression_index {
                    if let Some(comma) = tokens.value_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        self.write_token(&tokens.r#do);
        self.write_block(generic_for.get_block());
        self.write_token(&tokens.end);
    }

    fn write_if_statement_with_tokens(
        &mut self,
        if_statement: &IfStatement,
        tokens: &IfStatementTokens,
    ) {
        let mut branches = if_statement.iter_branches();
        if let Some(branch) = branches.next() {
            self.write_token(&tokens.r#if);
            self.write_expression(branch.get_condition());
            self.write_token(&tokens.then);
            self.write_block(branch.get_block());

            for branch in branches {
                if let Some(tokens) = branch.get_tokens() {
                    self.write_if_branch_with_tokens(branch, tokens);
                } else {
                    self.write_if_branch_with_tokens(
                        branch,
                        &self.generate_if_branch_tokens(branch),
                    );
                }
            }

            if let Some(else_block) = if_statement.get_else_block() {
                if let Some(token) = &tokens.r#else {
                    self.write_token(token);
                } else {
                    self.write_symbol("else");
                }
                self.write_block(else_block);
            }

            self.write_token(&tokens.end);
        }
    }

    fn write_if_branch_with_tokens(&mut self, branch: &IfBranch, tokens: &IfBranchTokens) {
        self.write_token(&tokens.elseif);
        self.write_expression(branch.get_condition());
        self.write_token(&tokens.then);
        self.write_block(branch.get_block());
    }

    fn write_local_assign_with_tokens(
        &mut self,
        assign: &LocalAssignStatement,
        tokens: &LocalAssignTokens,
    ) {
        self.write_token(&tokens.local);
        let last_variable_index = assign.variables_len().saturating_sub(1);
        assign
            .iter_variables()
            .enumerate()
            .for_each(|(i, identifier)| {
                self.write_identifier(identifier);
                if i < last_variable_index {
                    if let Some(comma) = tokens.variable_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        if assign.has_values() {
            if let Some(token) = &tokens.equal {
                self.write_token(token);
            } else {
                self.write_symbol("=");
            }
            let last_value_index = assign.values_len().saturating_sub(1);
            assign.iter_values().enumerate().for_each(|(i, value)| {
                self.write_expression(value);
                if i < last_value_index {
                    if let Some(comma) = tokens.value_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });
        }
    }

    fn write_local_function_with_tokens(
        &mut self,
        function: &LocalFunctionStatement,
        tokens: &LocalFunctionTokens,
    ) {
        self.write_token(&tokens.local);
        self.write_token(&tokens.function);
        self.write_identifier(function.get_identifier());

        self.write_token(&tokens.opening_parenthese);

        let parameter_count = function.parameters_count();
        let last_parameter_index = parameter_count.saturating_sub(1);
        function
            .iter_parameters()
            .enumerate()
            .for_each(|(i, param)| {
                self.write_identifier(param);
                if i < last_parameter_index {
                    if let Some(comma) = tokens.parameter_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        if function.is_variadic() {
            if function.has_parameters() {
                if let Some(comma) = tokens.parameter_commas.get(last_parameter_index) {
                    self.write_token(comma);
                } else {
                    self.write_symbol(",");
                }
            }

            if let Some(token) = &tokens.variable_arguments {
                self.write_token(token);
            } else {
                self.write_symbol("...");
            }
        }

        self.write_token(&tokens.closing_parenthese);

        self.write_block(function.get_block());
        self.write_token(&tokens.end);
    }

    fn write_numeric_for_with_tokens(
        &mut self,
        numeric_for: &NumericForStatement,
        tokens: &NumericForTokens,
    ) {
        self.write_token(&tokens.r#for);
        self.write_identifier(numeric_for.get_identifier());
        self.write_token(&tokens.equal);
        self.write_expression(numeric_for.get_start());
        self.write_token(&tokens.end_comma);
        self.write_expression(numeric_for.get_end());

        if let Some(step) = numeric_for.get_step() {
            if let Some(comma) = &tokens.step_comma {
                self.write_token(comma);
            } else {
                self.write_symbol(",");
            }
            self.write_expression(step);
        }

        self.write_token(&tokens.r#do);
        self.write_block(numeric_for.get_block());
        self.write_token(&tokens.end);
    }

    fn write_repeat_with_tokens(&mut self, repeat: &RepeatStatement, tokens: &RepeatTokens) {
        self.write_token(&tokens.repeat);
        self.write_block(repeat.get_block());
        self.write_token(&tokens.until);
        self.write_expression(repeat.get_condition());
    }

    fn write_while_with_tokens(&mut self, while_statement: &WhileStatement, tokens: &WhileTokens) {
        self.write_token(&tokens.r#while);
        self.write_expression(while_statement.get_condition());
        self.write_token(&tokens.r#do);
        self.write_block(while_statement.get_block());
        self.write_token(&tokens.end);
    }

    fn write_function_with_tokens(
        &mut self,
        function: &FunctionExpression,
        tokens: &FunctionExpressionTokens,
    ) {
        self.write_token(&tokens.function);
        self.write_token(&tokens.opening_parenthese);

        let parameter_count = function.parameters_count();
        let last_parameter_index = parameter_count.saturating_sub(1);
        function
            .iter_parameters()
            .enumerate()
            .for_each(|(i, param)| {
                self.write_identifier(param);
                if i < last_parameter_index {
                    if let Some(comma) = tokens.parameter_commas.get(i) {
                        self.write_token(comma);
                    } else {
                        self.write_symbol(",");
                    }
                }
            });

        if function.is_variadic() {
            if function.has_parameters() {
                if let Some(comma) = tokens.parameter_commas.get(last_parameter_index) {
                    self.write_token(comma);
                } else {
                    self.write_symbol(",");
                }
            }

            if let Some(token) = &tokens.variable_arguments {
                self.write_token(token);
            } else {
                self.write_symbol("...");
            }
        }

        self.write_token(&tokens.closing_parenthese);

        self.write_block(function.get_block());
        self.write_token(&tokens.end);
    }

    fn write_interpolated_string_with_tokens(
        &mut self,
        interpolated_string: &InterpolatedStringExpression,
        tokens: &InterpolatedStringTokens,
    ) {
        self.write_token(&tokens.opening_tick);

        for segment in interpolated_string.iter_segments() {
            match segment {
                InterpolationSegment::String(string_segment) => {
                    if let Some(token) = string_segment.get_token() {
                        self.write_token(token);
                    } else {
                        self.write_symbol(&utils::write_interpolated_string_segment(string_segment))
                    }
                }
                InterpolationSegment::Value(value) => {
                    if let Some(tokens) = value.get_tokens() {
                        self.write_string_value_segment_with_tokens(value, tokens);
                    } else {
                        self.write_string_value_segment_with_tokens(
                            value,
                            &self.generate_string_value_segment_tokens(value),
                        );
                    }
                }
            }
        }

        self.write_token(&tokens.closing_tick);
    }

    fn write_string_value_segment_with_tokens(
        &mut self,
        value: &ValueSegment,
        tokens: &ValueSegmentTokens,
    ) {
        self.write_token(&tokens.opening_brace);
        let expression = value.get_expression();
        if self.output.ends_with('{') {
            match expression {
                Expression::Table(table) => {
                    if let Some(tokens) = table.get_tokens() {
                        if let Some(first_trivia) =
                            tokens.opening_brace.iter_leading_trivia().next()
                        {
                            let trivia_str = first_trivia.read(self.original_code);
                            if trivia_str.is_empty() {
                                self.output.push(' ');
                            }
                        }
                    } else {
                        self.output.push(' ');
                    }
                }
                _ => {}
            }
        }
        self.write_expression(expression);
        self.write_token(&tokens.closing_brace);
    }

    fn generate_block_tokens(&self, _block: &Block) -> BlockTokens {
        BlockTokens {
            semicolons: Vec::new(),
            last_semicolon: None,
        }
    }

    fn generate_assign_tokens(&self, assign: &AssignStatement) -> AssignTokens {
        AssignTokens {
            equal: Token::from_content("="),
            variable_commas: intersect_with_token(comma_token(), assign.variables_len()),
            value_commas: intersect_with_token(comma_token(), assign.values_len()),
        }
    }

    fn generate_do_tokens(&self, _do_statement: &DoStatement) -> DoTokens {
        DoTokens {
            r#do: Token::from_content("do"),
            end: Token::from_content("end"),
        }
    }

    fn generate_compound_assign_tokens(
        &self,
        assign: &CompoundAssignStatement,
    ) -> CompoundAssignTokens {
        CompoundAssignTokens {
            operator: Token::from_content(assign.get_operator().to_str()),
        }
    }

    fn generate_generic_for_tokens(&self, generic_for: &GenericForStatement) -> GenericForTokens {
        GenericForTokens {
            r#for: Token::from_content("for"),
            r#in: Token::from_content("in"),
            r#do: Token::from_content("do"),
            end: Token::from_content("end"),
            identifier_commas: intersect_with_token(comma_token(), generic_for.identifiers_len()),
            value_commas: intersect_with_token(comma_token(), generic_for.expressions_len()),
        }
    }

    fn generate_if_statement_tokens(&self, if_statement: &IfStatement) -> IfStatementTokens {
        IfStatementTokens {
            r#if: Token::from_content("if"),
            then: Token::from_content("then"),
            end: Token::from_content("end"),
            r#else: if_statement
                .get_else_block()
                .map(|_| Token::from_content("else")),
        }
    }

    fn generate_if_branch_tokens(&self, _branch: &IfBranch) -> IfBranchTokens {
        IfBranchTokens {
            elseif: Token::from_content("elseif"),
            then: Token::from_content("then"),
        }
    }

    fn generate_function_statement_tokens(
        &self,
        function: &FunctionStatement,
    ) -> FunctionStatementTokens {
        FunctionStatementTokens {
            function: Token::from_content("function"),
            opening_parenthese: Token::from_content("("),
            closing_parenthese: Token::from_content(")"),
            end: Token::from_content("end"),
            parameter_commas: intersect_with_token(
                comma_token(),
                function.parameters_count() + usize::from(function.is_variadic()),
            ),
            variable_arguments: if function.is_variadic() {
                Some(Token::from_content("..."))
            } else {
                None
            },
        }
    }

    fn generate_function_name_tokens(&self, name: &FunctionName) -> FunctionNameTokens {
        FunctionNameTokens {
            periods: iter::repeat_with(|| Token::from_content("."))
                .take(name.get_field_names().len())
                .collect(),
            colon: name.get_method().map(|_| Token::from_content(":")),
        }
    }

    fn generate_return_tokens(&self, return_statement: &ReturnStatement) -> ReturnTokens {
        ReturnTokens {
            r#return: Token::from_content("return")
                .with_trailing_trivia(TriviaKind::Whitespace.with_content(" ")),
            commas: intersect_with_token(comma_token(), return_statement.len()),
        }
    }

    fn generate_local_assign_tokens(&self, assign: &LocalAssignStatement) -> LocalAssignTokens {
        LocalAssignTokens {
            local: Token::from_content("local"),
            equal: if assign.has_values() {
                Some(Token::from_content("="))
            } else {
                None
            },
            variable_commas: intersect_with_token(comma_token(), assign.variables_len()),
            value_commas: intersect_with_token(comma_token(), assign.values_len()),
        }
    }

    fn generate_local_function_tokens(
        &self,
        function: &LocalFunctionStatement,
    ) -> LocalFunctionTokens {
        LocalFunctionTokens {
            local: Token::from_content("local"),
            function: Token::from_content("function"),
            opening_parenthese: Token::from_content("("),
            closing_parenthese: Token::from_content(")"),
            end: Token::from_content("end"),
            parameter_commas: intersect_with_token(
                comma_token(),
                function.parameters_count() + usize::from(function.is_variadic()),
            ),
            variable_arguments: if function.is_variadic() {
                Some(Token::from_content("..."))
            } else {
                None
            },
        }
    }

    fn generate_numeric_for_tokens(&self, numeric_for: &NumericForStatement) -> NumericForTokens {
        NumericForTokens {
            r#for: Token::from_content("for"),
            equal: Token::from_content("="),
            r#do: Token::from_content("do"),
            end: Token::from_content("end"),
            end_comma: Token::from_content(","),
            step_comma: numeric_for.get_step().map(|_| Token::from_content(",")),
        }
    }

    fn generate_repeat_tokens(&self, _repeat: &RepeatStatement) -> RepeatTokens {
        RepeatTokens {
            repeat: Token::from_content("repeat"),
            until: Token::from_content("until"),
        }
    }

    fn generate_while_tokens(&self, _while_statement: &WhileStatement) -> WhileTokens {
        WhileTokens {
            r#while: Token::from_content("while"),
            r#do: Token::from_content("do"),
            end: Token::from_content("end"),
        }
    }

    fn generate_function_tokens(&self, function: &FunctionExpression) -> FunctionExpressionTokens {
        FunctionExpressionTokens {
            function: Token::from_content("function"),
            opening_parenthese: Token::from_content("("),
            closing_parenthese: Token::from_content(")"),
            end: Token::from_content("end"),
            parameter_commas: intersect_with_token(
                comma_token(),
                function.parameters_count() + usize::from(function.is_variadic()),
            ),
            variable_arguments: if function.is_variadic() {
                Some(Token::from_content("..."))
            } else {
                None
            },
        }
    }

    fn generate_function_call_tokens(&self, call: &FunctionCall) -> FunctionCallTokens {
        FunctionCallTokens {
            colon: call.get_method().map(|_| Token::from_content(":")),
        }
    }

    fn generate_field_token(&self, _field: &FieldExpression) -> Token {
        Token::from_content(".")
    }

    fn generate_index_tokens(&self, _index: &IndexExpression) -> IndexExpressionTokens {
        IndexExpressionTokens {
            opening_bracket: Token::from_content("["),
            closing_bracket: Token::from_content("]"),
        }
    }

    fn generate_if_tokens(&self, _if_expression: &IfExpression) -> IfExpressionTokens {
        IfExpressionTokens {
            r#if: Token::from_content("if"),
            then: Token::from_content("then"),
            r#else: Token::from_content("else"),
        }
    }

    fn generate_if_expression_branch_tokens(
        &self,
        _branch: &ElseIfExpressionBranch,
    ) -> ElseIfExpressionBranchTokens {
        ElseIfExpressionBranchTokens {
            elseif: Token::from_content("elseif"),
            then: Token::from_content("then"),
        }
    }

    fn generate_table_tokens(&self, table: &TableExpression) -> TableTokens {
        TableTokens {
            opening_brace: Token::from_content("{"),
            closing_brace: Token::from_content("}"),
            separators: intersect_with_token(comma_token(), table.len()),
        }
    }

    fn generate_table_field_tokens(&self, _entry: &TableFieldEntry) -> Token {
        Token::from_content("=")
    }

    fn generate_table_index_tokens(&self, _entry: &TableIndexEntry) -> TableIndexEntryTokens {
        TableIndexEntryTokens {
            opening_bracket: Token::from_content("["),
            closing_bracket: Token::from_content("]"),
            equal: Token::from_content("="),
        }
    }

    fn generate_tuple_arguments_tokens(&self, arguments: &TupleArguments) -> TupleArgumentsTokens {
        TupleArgumentsTokens {
            opening_parenthese: Token::from_content("("),
            closing_parenthese: Token::from_content(")"),
            commas: intersect_with_token(comma_token(), arguments.len()),
        }
    }

    fn generate_parenthese_tokens(&self, _parenthese: &ParentheseExpression) -> ParentheseTokens {
        ParentheseTokens {
            left_parenthese: Token::from_content("("),
            right_parenthese: Token::from_content(")"),
        }
    }

    fn generate_interpolated_string_tokens(
        &self,
        _interpolated_string: &InterpolatedStringExpression,
    ) -> InterpolatedStringTokens {
        InterpolatedStringTokens {
            opening_tick: Token::from_content("`"),
            closing_tick: Token::from_content("`"),
        }
    }

    fn generate_string_value_segment_tokens(
        &self,
        _value_segment: &ValueSegment,
    ) -> ValueSegmentTokens {
        ValueSegmentTokens {
            opening_brace: Token::from_content("{"),
            closing_brace: Token::from_content("}"),
        }
    }

    fn write_symbol(&mut self, symbol: &str) {
        if self.currently_commenting {
            self.uncomment();
        } else if self.needs_space(symbol.chars().next().expect("symbol cannot be empty")) {
            self.output.push(' ');
        }
        self.push_str(symbol);
    }

    #[inline]
    fn needs_space(&self, next_character: char) -> bool {
        is_ending_relevant_for_spacing(next_character)
            && if let Some(last) = self.output.chars().last() {
                is_relevant_for_spacing(last, next_character)
            } else {
                false
            }
    }

    #[inline]
    fn uncomment(&mut self) {
        self.output.push('\n');
        self.current_line += 1;
        self.currently_commenting = false;
    }
}

fn is_single_line_comment(content: &str) -> bool {
    let is_multiline_comment = content.starts_with("--[") && {
        if let Some((closing_bracket_index, _)) =
            content.chars().skip(3).enumerate().find(|(_, c)| *c == '[')
        {
            content
                .get(3..closing_bracket_index)
                .map(|substring| substring.chars().all(|c| c == '='))
                .unwrap_or(true)
        } else {
            false
        }
    };

    !is_multiline_comment
}

#[inline]
fn is_ending_relevant_for_spacing(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '[' | ']' | '.')
}

#[inline]
fn is_relevant_for_spacing(ending_character: char, next_character: char) -> bool {
    match ending_character {
        '0'..='9' => matches!(next_character, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '.'),
        'A'..='Z' | 'a'..='z' | '_' => {
            next_character.is_ascii_alphanumeric() || next_character == '_'
        }
        '-' => next_character == '-',
        '[' => next_character == '[',
        ']' => next_character == ']',
        '.' => matches!(next_character, '.' | '0'..='9'),
        _ => false,
    }
}

#[inline]
fn comma_token() -> Token {
    Token::from_content(",").with_trailing_trivia(TriviaKind::Whitespace.with_content(" "))
}

impl<'a> LuaGenerator for TokenBasedLuaGenerator<'a> {
    fn into_string(self) -> String {
        self.output
    }

    fn write_block(&mut self, block: &Block) {
        if let Some(tokens) = block.get_tokens() {
            self.write_block_with_tokens(block, tokens);
        } else {
            self.write_block_with_tokens(block, &self.generate_block_tokens(block));
        }
    }

    fn write_assign_statement(&mut self, assign: &AssignStatement) {
        if let Some(tokens) = assign.get_tokens() {
            self.write_assign_with_tokens(assign, tokens);
        } else {
            self.write_assign_with_tokens(assign, &self.generate_assign_tokens(assign));
        }
    }

    fn write_do_statement(&mut self, do_statement: &DoStatement) {
        if let Some(tokens) = do_statement.get_tokens() {
            self.write_do_with_tokens(do_statement, tokens);
        } else {
            self.write_do_with_tokens(do_statement, &self.generate_do_tokens(do_statement));
        }
    }

    fn write_compound_assign(&mut self, assign: &CompoundAssignStatement) {
        if let Some(tokens) = assign.get_tokens() {
            self.write_compound_assign_with_tokens(assign, tokens);
        } else {
            self.write_compound_assign_with_tokens(
                assign,
                &self.generate_compound_assign_tokens(assign),
            );
        }
    }

    fn write_generic_for(&mut self, generic_for: &GenericForStatement) {
        if let Some(tokens) = generic_for.get_tokens() {
            self.write_generic_for_with_tokens(generic_for, tokens);
        } else {
            self.write_generic_for_with_tokens(
                generic_for,
                &self.generate_generic_for_tokens(generic_for),
            );
        }
    }

    fn write_if_statement(&mut self, if_statement: &IfStatement) {
        if let Some(tokens) = if_statement.get_tokens() {
            self.write_if_statement_with_tokens(if_statement, tokens);
        } else {
            self.write_if_statement_with_tokens(
                if_statement,
                &self.generate_if_statement_tokens(if_statement),
            );
        }
    }

    fn write_function_statement(&mut self, function: &FunctionStatement) {
        if let Some(tokens) = function.get_tokens() {
            self.write_function_statement_with_tokens(function, tokens);
        } else {
            self.write_function_statement_with_tokens(
                function,
                &self.generate_function_statement_tokens(function),
            );
        }
    }

    fn write_last_statement(&mut self, statement: &LastStatement) {
        match statement {
            LastStatement::Break(token) => {
                if let Some(token) = token {
                    self.write_token(token);
                } else {
                    self.write_symbol("break");
                }
            }
            LastStatement::Continue(token) => {
                if let Some(token) = token {
                    self.write_token(token);
                } else {
                    self.write_symbol("continue");
                }
            }
            LastStatement::Return(return_statement) => {
                if let Some(tokens) = return_statement.get_tokens() {
                    self.write_return_with_tokens(return_statement, tokens);
                } else {
                    self.write_return_with_tokens(
                        return_statement,
                        &self.generate_return_tokens(return_statement),
                    );
                }
            }
        }
    }

    fn write_local_assign(&mut self, assign: &LocalAssignStatement) {
        if let Some(tokens) = assign.get_tokens() {
            self.write_local_assign_with_tokens(assign, tokens);
        } else {
            self.write_local_assign_with_tokens(assign, &self.generate_local_assign_tokens(assign));
        }
    }

    fn write_local_function(&mut self, function: &LocalFunctionStatement) {
        if let Some(tokens) = function.get_tokens() {
            self.write_local_function_with_tokens(function, tokens);
        } else {
            self.write_local_function_with_tokens(
                function,
                &self.generate_local_function_tokens(function),
            );
        }
    }

    fn write_numeric_for(&mut self, numeric_for: &NumericForStatement) {
        if let Some(tokens) = numeric_for.get_tokens() {
            self.write_numeric_for_with_tokens(numeric_for, tokens);
        } else {
            self.write_numeric_for_with_tokens(
                numeric_for,
                &self.generate_numeric_for_tokens(numeric_for),
            );
        }
    }

    fn write_repeat_statement(&mut self, repeat: &RepeatStatement) {
        if let Some(tokens) = repeat.get_tokens() {
            self.write_repeat_with_tokens(repeat, tokens);
        } else {
            self.write_repeat_with_tokens(repeat, &self.generate_repeat_tokens(repeat));
        }
    }

    fn write_while_statement(&mut self, while_statement: &WhileStatement) {
        if let Some(tokens) = while_statement.get_tokens() {
            self.write_while_with_tokens(while_statement, tokens);
        } else {
            self.write_while_with_tokens(
                while_statement,
                &self.generate_while_tokens(while_statement),
            );
        }
    }

    fn write_expression(&mut self, expression: &Expression) {
        use Expression::*;
        match expression {
            Binary(binary) => self.write_binary_expression(binary),
            Call(call) => self.write_function_call(call),
            False(token) => {
                if let Some(token) = token {
                    self.write_token(token);
                } else {
                    self.write_symbol("false");
                }
            }
            Field(field) => self.write_field(field),
            Function(function) => self.write_function(function),
            Identifier(identifier) => self.write_identifier(identifier),
            If(if_expression) => self.write_if_expression(if_expression),
            Index(index) => self.write_index(index),
            Nil(token) => {
                if let Some(token) = token {
                    self.write_token(token);
                } else {
                    self.write_symbol("nil");
                }
            }
            Number(number) => self.write_number(number),
            Parenthese(parenthese) => self.write_parenthese(parenthese),
            String(string) => self.write_string(string),
            InterpolatedString(interpolated_string) => {
                self.write_interpolated_string(interpolated_string);
            }
            Table(table) => self.write_table(table),
            True(token) => {
                if let Some(token) = token {
                    self.write_token(token);
                } else {
                    self.write_symbol("true");
                }
            }
            Unary(unary) => self.write_unary_expression(unary),
            VariableArguments(token) => {
                if let Some(token) = token {
                    self.write_token(token);
                } else {
                    self.write_symbol("...");
                }
            }
        }
    }

    fn write_binary_expression(&mut self, binary: &BinaryExpression) {
        let operator = binary.operator();
        let left = binary.left();
        let right = binary.right();

        if operator.left_needs_parentheses(left) {
            self.write_symbol("(");
            self.write_expression(left);
            self.write_symbol(")");
        } else {
            self.write_expression(left);
        }

        if let Some(operator) = binary.get_token() {
            self.write_token(operator);
        } else {
            self.write_token(&Token::from_content(binary.operator().to_str()));
        }

        if operator.right_needs_parentheses(right) {
            self.write_symbol("(");
            self.write_expression(right);
            self.write_symbol(")");
        } else {
            self.write_expression(right);
        }
    }

    fn write_unary_expression(&mut self, unary: &UnaryExpression) {
        if let Some(operator) = unary.get_token() {
            self.write_token(operator);
        } else {
            self.write_token(&Token::from_content(unary.operator().to_str()));
        }

        let expression = unary.get_expression();
        match expression {
            Expression::Binary(binary) if !binary.operator().precedes_unary_expression() => {
                self.write_symbol("(");
                self.write_expression(expression);
                self.write_symbol(")");
            }
            _ => self.write_expression(expression),
        }
    }

    fn write_function(&mut self, function: &FunctionExpression) {
        if let Some(tokens) = function.get_tokens() {
            self.write_function_with_tokens(function, tokens);
        } else {
            self.write_function_with_tokens(function, &self.generate_function_tokens(function));
        }
    }

    fn write_function_call(&mut self, call: &FunctionCall) {
        if let Some(tokens) = call.get_tokens() {
            self.write_function_call_with_tokens(call, tokens);
        } else {
            self.write_function_call_with_tokens(call, &self.generate_function_call_tokens(call));
        }
    }

    fn write_field(&mut self, field: &FieldExpression) {
        if let Some(token) = field.get_token() {
            self.write_field_with_token(field, token);
        } else {
            self.write_field_with_token(field, &self.generate_field_token(field));
        }
    }

    fn write_index(&mut self, index: &IndexExpression) {
        if let Some(tokens) = index.get_tokens() {
            self.write_index_with_tokens(index, tokens);
        } else {
            self.write_index_with_tokens(index, &self.generate_index_tokens(index));
        }
    }

    fn write_if_expression(&mut self, if_expression: &IfExpression) {
        if let Some(token) = if_expression.get_tokens() {
            self.write_if_expression_with_token(if_expression, token);
        } else {
            self.write_if_expression_with_token(
                if_expression,
                &self.generate_if_tokens(if_expression),
            );
        }
    }

    fn write_table(&mut self, table: &TableExpression) {
        if let Some(tokens) = table.get_tokens() {
            self.write_table_with_tokens(table, tokens);
        } else {
            self.write_table_with_tokens(table, &self.generate_table_tokens(table));
        }
    }

    fn write_table_entry(&mut self, entry: &TableEntry) {
        match entry {
            TableEntry::Field(entry) => {
                if let Some(tokens) = entry.get_token() {
                    self.write_table_field_with_tokens(entry, tokens);
                } else {
                    self.write_table_field_with_tokens(
                        entry,
                        &self.generate_table_field_tokens(entry),
                    );
                }
            }
            TableEntry::Index(entry) => {
                if let Some(tokens) = entry.get_tokens() {
                    self.write_table_index_with_tokens(entry, tokens);
                } else {
                    self.write_table_index_with_tokens(
                        entry,
                        &self.generate_table_index_tokens(entry),
                    );
                }
            }
            TableEntry::Value(expression) => self.write_expression(expression),
        }
    }

    fn write_number(&mut self, number: &NumberExpression) {
        if let Some(token) = number.get_token() {
            self.write_token(token);
        } else {
            self.write_token(&Token::from_content(utils::write_number(number)));
        }
    }

    fn write_tuple_arguments(&mut self, arguments: &TupleArguments) {
        if let Some(tokens) = arguments.get_tokens() {
            self.write_tuple_arguments_with_tokens(arguments, tokens);
        } else {
            self.write_tuple_arguments_with_tokens(
                arguments,
                &self.generate_tuple_arguments_tokens(arguments),
            );
        }
    }

    fn write_string(&mut self, string: &StringExpression) {
        if let Some(token) = string.get_token() {
            self.write_token(token);
        } else {
            self.write_symbol(&utils::write_string(string));
        }
    }

    fn write_interpolated_string(&mut self, interpolated_string: &InterpolatedStringExpression) {
        if let Some(tokens) = interpolated_string.get_tokens() {
            self.write_interpolated_string_with_tokens(interpolated_string, tokens);
        } else {
            self.write_interpolated_string_with_tokens(
                interpolated_string,
                &self.generate_interpolated_string_tokens(interpolated_string),
            );
        }
    }

    fn write_identifier(&mut self, identifier: &Identifier) {
        if let Some(token) = identifier.get_token() {
            let name_in_token = token.read(self.original_code);

            if name_in_token == identifier.get_name() {
                self.write_token(token);
            } else {
                let mut new_token = token.clone();
                new_token.replace_with_content(identifier.get_name().clone());
                self.write_token(&new_token);
            }
        } else {
            let name = identifier.get_name();
            self.write_symbol(name);
        }
    }

    fn write_parenthese(&mut self, parenthese: &ParentheseExpression) {
        if let Some(tokens) = parenthese.get_tokens() {
            self.write_parenthese_with_tokens(parenthese, tokens);
        } else {
            self.write_parenthese_with_tokens(
                parenthese,
                &self.generate_parenthese_tokens(parenthese),
            );
        }
    }
}

fn intersect_with_token(token: Token, list_length: usize) -> Vec<Token> {
    iter::repeat_with(|| token.clone())
        .take(list_length.saturating_sub(1))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_output {
        ($($name:ident => $code:literal),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let parser = crate::Parser::default().preserve_tokens();
                    let block = parser.parse($code)
                        .expect(&format!("failed to parse `{}`", $code));

                    let mut generator = TokenBasedLuaGenerator::new($code);

                    generator.write_block(&block);

                    let output = generator.into_string();

                    assert_eq!($code, &output);
                }
            )*

            mod without_tokens {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let parser = crate::Parser::default();
                        let block = parser.parse($code)
                            .expect(&format!("failed to parse `{}`", $code));

                        let mut generator = TokenBasedLuaGenerator::new($code);

                        generator.write_block(&block);

                        let output = generator.into_string();

                        let parsed_output_block = parser.parse(&output)
                            .expect(&format!("failed to parse generated code `{}`", &output));

                        pretty_assertions::assert_eq!(block, parsed_output_block);
                    }
                )*
            }
        };
    }

    test_output!(
        // statements
        assign => "var = true",
        assign_multiple => "var, var2 =\n\ttrue,\tfalse\n",
        empty_do => "do end\n",
        nested_do => "do\n    do end\nend\n",
        call_without_arguments => "call()",
        call_print => "print('hi')",
        call_print_with_string_argument => "print 'hi' -- no parentheses",
        call_function_with_table_multiline => "process {\n\targ = true,\n\tflag = false,\n}\n",
        call_method_without_arguments => "foo:bar()",
        call_method_with_arguments => "foo:bar(true, false)",
        call_string_format => "('foo'):rep(3)",
        call_math_floor => "math.floor(value)",
        call_with_index => "object[ key ](i)",
        compound_increment => "i += 1\n",
        empty_function_declaration => "function process()\nend",
        empty_static_function_declaration => "function Class .new()\nend",
        empty_method_function_declaration => "function Class : process()\nend",
        empty_nested_method_function_declaration => "function Class . foo.bar : help ()\nend",
        empty_function_declaration_with_params => "function process(a, b --[[ optional ]]) end",
        empty_variadic_function_declaration => "function process (...) end",
        empty_variadic_function_declaration_with_one_param => "function format (str, ... --[[ optional strings ]]) end",
        variadic_function_returns => "function identity(...)\n\treturn ...\nend\n",
        empty_generic_for => "for key, value in pairs(result) do\n\t-- help\nend",
        empty_generic_for_key_only => "for key in pairs(dict) do end",
        generic_for_with_next => "for key,value in next, dict do\n\tprint(key, value)\nend\n",
        empty_if => "if true then\nend",
        if_condition_return => "if condition then\n\treturn\nend",
        empty_if_with_empty_elseif => "if true then\nelseif false then\nend\n",
        empty_if_with_two_empty_elseif => "if a then\nelseif b then\nelseif c then\n\tprint(c)\nend\n",
        empty_if_with_empty_else => "if true then\nelse\nend\n",
        empty_if_with_else_block => "if true then\n\t-- help\nelse\n\treturn\nend\n",
        declare_one_variable => "local var\n",
        declare_two_variables => "local var, var2\n",
        local_assign_one_variable => "local var = true",
        local_assign_two_variables => "local var, var2 = true, false",
        local_empty_function => "local function process()\nend",
        local_empty_variadic_function => "local function process(...)\nend",
        local_empty_function_with_one_argument => "local function process( a )\nend",
        local_empty_function_with_two_arguments => "local function process(a, b)\nend",
        local_empty_variadic_function_with_two_arguments => "local function process(a, b, ...)\nend",
        local_identity_function => "local function identity(...)\n\treturn ...\nend",
        empty_numeric_for => "for i = 1, final do\nend\n",
        empty_numeric_for_with_step => "for i = 1, final, step do\nend\n",
        numeric_for => "for i = 1, #list do\n\tprocess(list[i])\nend",
        empty_repeat => "repeat until false",
        repeat_break_immediately => "repeat break until false",
        empty_while => "while true do end",
        while_break_immediately => "while true do\n\tbreak\nend",

        // last statements
        break_with_comment => "break -- exit loop",
        continue_with_comment => "continue -- skip to next iteration",
        empty_return => "return\n",

        // expressions
        return_true => "return true",
        return_false => "return false",
        return_nil => "return nil",
        return_single_quote_string => "return 'ok'",
        return_double_quote_string => "return \"ok\"",
        return_identifier => "return var",
        return_bracket_string => "return [[   [ok]   ]]",
        return_empty_interpolated_string => "return ``",
        return_interpolated_string_escape_curly_brace => "return `Open: \\{`",
        return_interpolated_string_followed_by_comment => "return `ok` -- comment",
        return_interpolated_string_with_true_value => "return `{ true }`",
        return_interpolated_string_with_true_value_and_prefix => "return `Result = { true }`",
        return_interpolated_string_with_true_value_and_suffix => "return `{ variable } !`",
        return_interpolated_string_with_various_segments => "return `Variable = { variable } ({ --[[len]] #variable })` -- display",
        return_empty_table => "return { }",
        return_table_with_field => "return { field = {} }",
        return_table_with_index => "return { [field] = {} }",
        return_list_of_one_element => "return { true, }",
        return_list_of_two_elements => "return { true, false }",
        return_mixed_table => "return { true, field = false, [\"hello\"] = true }",
        return_parenthese_call => "return ( call() )",
        return_variable_arguments => "return ...",
        return_unary_minus => "return - number",
        return_unary_length => "return #list\n",
        return_unary_not => "return not condition\n",
        return_binary_and => "return a and b",
        return_binary_or => "return a or b",
        return_binary_plus => "return 10 + 15",
        return_empty_function => "return function() end",
        return_empty_variadic_function => "return function(...)\nend",
        return_empty_function_with_one_argument => "return function( a )\nend",
        return_empty_function_with_two_arguments => "return function(a, b)\nend",
        return_empty_variadic_function_with_two_arguments => "return function(a, b, ...)\nend",
        return_identity_function => "return function(...)\n\treturn ...\nend",
        return_field => "return math.huge",
        return_field_ending_with_number => "return UDim2.new",
        return_field_split_on_lines => "return value.\n\tproperty\n\t.name",
    );

    #[test]
    fn inserts_a_new_line_after_a_comment_for_a_token() {
        let statement = RepeatStatement::new(Block::default(), true).with_tokens(RepeatTokens {
            repeat: Token::from_content("repeat")
                .with_trailing_trivia(TriviaKind::Comment.with_content("-- hello")),
            until: Token::from_content("until"),
        });

        let mut generator = TokenBasedLuaGenerator::new("");

        generator.write_repeat_statement(&statement);

        let output = generator.into_string();

        crate::Parser::default()
            .parse(&output)
            .unwrap_or_else(|_| panic!("failed to parse generated code `{}`", &output));
    }

    #[test]
    fn inserts_a_new_line_after_custom_added_comments() {
        let code = "call(a--comment\n\t,b\n)";
        let mut block = crate::Parser::default()
            .preserve_tokens()
            .parse(code)
            .unwrap();

        let call = match block.iter_mut_statements().last().unwrap() {
            Statement::Call(call) => call,
            _ => panic!("unexpected statement"),
        };
        let tuple = match call.mutate_arguments() {
            Arguments::Tuple(tuple) => tuple,
            _ => panic!("unexpected arguments"),
        };
        let mut tokens = tuple.get_tokens().unwrap().clone();

        tuple.iter_mut_values().for_each(|value| match value {
            Expression::Identifier(identifier) => {
                let name = identifier.mutate_name();
                let new_token = {
                    Token::from_content(name.to_owned())
                        .with_trailing_trivia(TriviaKind::Comment.with_content("--new comment"))
                };
                identifier.set_token(new_token);
            }
            _ => panic!("unexpected expression"),
        });

        // drop a comma and verify that the generator does not screw things up
        tokens.commas.pop();
        tuple.set_tokens(tokens);

        let mut generator = TokenBasedLuaGenerator::new(code);

        generator.write_block(&block);

        let output = generator.into_string();

        crate::Parser::default()
            .parse(&output)
            .unwrap_or_else(|_| panic!("failed to parse generated code `{}`", &output));

        insta::assert_snapshot!("inserts_a_new_line_after_custom_added_comments", output);
    }
}
