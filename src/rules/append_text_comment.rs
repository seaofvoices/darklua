use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::nodes::{
    Block, BlockTokens, DoTokens, FunctionBodyTokens, GenericForTokens, Identifier,
    IfStatementTokens, LastStatement, LocalAssignTokens, LocalFunctionTokens, NumericForTokens,
    ParentheseExpression, ParentheseTokens, Prefix, RepeatTokens, ReturnTokens, Statement, Token,
    TriviaKind, TypeDeclarationTokens, Variable, WhileTokens,
};
use crate::rules::{
    verify_property_collisions, verify_required_any_properties, Context, Rule, RuleConfiguration,
    RuleConfigurationError, RuleProcessResult, RuleProperties,
};

use super::{FlawlessRule, ShiftTokenLine};

pub const APPEND_TEXT_COMMENT_RULE_NAME: &str = "append_text_comment";

/// A rule to append a comment at the beginning or the end of each file.
#[derive(Debug, Default)]
pub struct AppendTextComment {
    text_value: OnceLock<Result<String, String>>,
    text_content: TextContent,
    location: AppendLocation,
}

impl AppendTextComment {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            text_value: Default::default(),
            text_content: TextContent::Value(value.into()),
            location: Default::default(),
        }
    }

    pub fn from_file_content(file_path: impl Into<PathBuf>) -> Self {
        Self {
            text_value: Default::default(),
            text_content: TextContent::FilePath(file_path.into()),
            location: Default::default(),
        }
    }

    pub fn at_end(mut self) -> Self {
        self.location = AppendLocation::End;
        self
    }

    fn text(&self, project_path: &Path) -> Result<String, String> {
        self.text_value
            .get_or_init(|| {
                match &self.text_content {
                    TextContent::None => Err("".to_owned()),
                    TextContent::Value(value) => Ok(value.clone()),
                    TextContent::FilePath(file_path) => {
                        fs::read_to_string(project_path.join(file_path)).map_err(|err| {
                            format!("unable to read file `{}`: {}", file_path.display(), err)
                        })
                    }
                }
                .map(|content| {
                    if content.is_empty() {
                        "".to_owned()
                    } else if content.contains('\n') {
                        let mut equal_count = 0;

                        let close_comment = loop {
                            let close_comment = format!("]{}]", "=".repeat(equal_count));
                            if !content.contains(&close_comment) {
                                break close_comment;
                            }
                            equal_count += 1;
                        };

                        format!(
                            "--[{}[\n{}\n{}",
                            "=".repeat(equal_count),
                            content,
                            close_comment
                        )
                    } else {
                        format!("--{}", content)
                    }
                })
            })
            .clone()
    }
}

impl Rule for AppendTextComment {
    fn process(&self, block: &mut Block, context: &Context) -> RuleProcessResult {
        let text = self.text(context.project_location())?;

        if text.is_empty() {
            return Ok(());
        }

        let shift_lines = text.lines().count();
        ShiftTokenLine::new(shift_lines as isize).flawless_process(block, context);

        match self.location {
            AppendLocation::Start => {
                if let Some(statement) = block.first_mut_statement() {
                    match statement {
                        Statement::Assign(assign_statement) => {
                            let variable = assign_statement
                                .iter_mut_variables()
                                .next()
                                .ok_or("an assign statement must have at least one variable")?;
                            self.location
                                .append_comment(variable_get_first_token(variable), text);
                        }
                        Statement::Do(do_statement) => {
                            if let Some(tokens) = do_statement.mutate_tokens() {
                                self.location.append_comment(&mut tokens.r#do, text);
                            } else {
                                let mut token = Token::from_content("do");
                                self.location.append_comment(&mut token, text);

                                do_statement.set_tokens(DoTokens {
                                    r#do: token,
                                    end: Token::from_content("end"),
                                });
                            }
                        }
                        Statement::Call(call) => {
                            self.location
                                .append_comment(prefix_get_first_token(call.mutate_prefix()), text);
                        }
                        Statement::CompoundAssign(compound_assign) => {
                            self.location.append_comment(
                                variable_get_first_token(compound_assign.mutate_variable()),
                                text,
                            );
                        }
                        Statement::Function(function) => {
                            if let Some(tokens) = function.mutate_tokens() {
                                self.location.append_comment(&mut tokens.function, text);
                            } else {
                                let mut token = Token::from_content("function");
                                self.location.append_comment(&mut token, text);

                                function.set_tokens(FunctionBodyTokens {
                                    function: token,
                                    opening_parenthese: Token::from_content("("),
                                    closing_parenthese: Token::from_content(")"),
                                    end: Token::from_content("end"),
                                    parameter_commas: Vec::new(),
                                    variable_arguments: None,
                                    variable_arguments_colon: None,
                                    return_type_colon: None,
                                });
                            }
                        }
                        Statement::GenericFor(generic_for) => {
                            if let Some(tokens) = generic_for.mutate_tokens() {
                                self.location.append_comment(&mut tokens.r#for, text);
                            } else {
                                let mut token = Token::from_content("for");
                                self.location.append_comment(&mut token, text);

                                generic_for.set_tokens(GenericForTokens {
                                    r#for: token,
                                    r#in: Token::from_content("in"),
                                    r#do: Token::from_content("do"),
                                    end: Token::from_content("end"),
                                    identifier_commas: Vec::new(),
                                    value_commas: Vec::new(),
                                });
                            }
                        }
                        Statement::If(if_statement) => {
                            if let Some(tokens) = if_statement.mutate_tokens() {
                                self.location.append_comment(&mut tokens.r#if, text);
                            } else {
                                let mut token = Token::from_content("if");
                                self.location.append_comment(&mut token, text);

                                if_statement.set_tokens(IfStatementTokens {
                                    r#if: token,
                                    then: Token::from_content("then"),
                                    end: Token::from_content("end"),
                                    r#else: None,
                                });
                            }
                        }
                        Statement::LocalAssign(local_assign) => {
                            if let Some(tokens) = local_assign.mutate_tokens() {
                                self.location.append_comment(&mut tokens.local, text);
                            } else {
                                let mut token = Token::from_content("local");
                                self.location.append_comment(&mut token, text);

                                local_assign.set_tokens(LocalAssignTokens {
                                    local: token,
                                    equal: None,
                                    variable_commas: Vec::new(),
                                    value_commas: Vec::new(),
                                });
                            }
                        }
                        Statement::LocalFunction(local_function) => {
                            if let Some(tokens) = local_function.mutate_tokens() {
                                self.location.append_comment(&mut tokens.local, text);
                            } else {
                                let mut token = Token::from_content("local");
                                self.location.append_comment(&mut token, text);

                                local_function.set_tokens(LocalFunctionTokens {
                                    local: token,
                                    function_body: FunctionBodyTokens {
                                        function: Token::from_content("function"),
                                        opening_parenthese: Token::from_content("("),
                                        closing_parenthese: Token::from_content(")"),
                                        end: Token::from_content("end"),
                                        parameter_commas: Vec::new(),
                                        variable_arguments: None,
                                        variable_arguments_colon: None,
                                        return_type_colon: None,
                                    },
                                });
                            }
                        }
                        Statement::NumericFor(numeric_for) => {
                            if let Some(tokens) = numeric_for.mutate_tokens() {
                                self.location.append_comment(&mut tokens.r#for, text);
                            } else {
                                let mut token = Token::from_content("for");
                                self.location.append_comment(&mut token, text);

                                numeric_for.set_tokens(NumericForTokens {
                                    r#for: token,
                                    equal: Token::from_content("="),
                                    r#do: Token::from_content("do"),
                                    end: Token::from_content("end"),
                                    end_comma: Token::from_content(","),
                                    step_comma: None,
                                });
                            }
                        }
                        Statement::Repeat(repeat) => {
                            if let Some(tokens) = repeat.mutate_tokens() {
                                self.location.append_comment(&mut tokens.repeat, text);
                            } else {
                                let mut token = Token::from_content("repeat");
                                self.location.append_comment(&mut token, text);

                                repeat.set_tokens(RepeatTokens {
                                    repeat: token,
                                    until: Token::from_content("until"),
                                });
                            }
                        }
                        Statement::While(while_statement) => {
                            if let Some(tokens) = while_statement.mutate_tokens() {
                                self.location.append_comment(&mut tokens.r#while, text);
                            } else {
                                let mut token = Token::from_content("while");
                                self.location.append_comment(&mut token, text);

                                while_statement.set_tokens(WhileTokens {
                                    r#while: token,
                                    r#do: Token::from_content("do"),
                                    end: Token::from_content("end"),
                                });
                            }
                        }
                        Statement::TypeDeclaration(type_declaration) => {
                            let is_exported = type_declaration.is_exported();
                            if let Some(tokens) = type_declaration.mutate_tokens() {
                                if is_exported {
                                    self.location.append_comment(
                                        tokens
                                            .export
                                            .get_or_insert_with(|| Token::from_content("export")),
                                        text,
                                    );
                                } else {
                                    self.location.append_comment(&mut tokens.r#type, text);
                                }
                            } else if is_exported {
                                let mut token = Token::from_content("export");
                                self.location.append_comment(&mut token, text);

                                type_declaration.set_tokens(TypeDeclarationTokens {
                                    r#type: Token::from_content("type"),
                                    equal: Token::from_content("="),
                                    export: Some(token),
                                });
                            } else {
                                let mut token = Token::from_content("type");
                                self.location.append_comment(&mut token, text);

                                type_declaration.set_tokens(TypeDeclarationTokens {
                                    r#type: token,
                                    equal: Token::from_content("="),
                                    export: None,
                                });
                            }
                        }
                    }
                } else if let Some(statement) = block.mutate_last_statement() {
                    match statement {
                        LastStatement::Break(token) => {
                            self.location.append_comment(
                                token.get_or_insert_with(|| Token::from_content("break")),
                                text,
                            );
                        }
                        LastStatement::Continue(token) => {
                            self.location.append_comment(
                                token.get_or_insert_with(|| Token::from_content("continue")),
                                text,
                            );
                        }
                        LastStatement::Return(return_statement) => {
                            if let Some(tokens) = return_statement.mutate_tokens() {
                                self.location.append_comment(&mut tokens.r#return, text);
                            } else {
                                let mut token = Token::from_content("return");
                                self.location.append_comment(&mut token, text);

                                return_statement.set_tokens(ReturnTokens {
                                    r#return: token,
                                    commas: Vec::new(),
                                });
                            }
                        }
                    }
                } else {
                    self.location.write_to_block(block, text);
                }
            }
            AppendLocation::End => {
                self.location.write_to_block(block, text);
            }
        }

        Ok(())
    }
}

fn variable_get_first_token(variable: &mut Variable) -> &mut Token {
    match variable {
        Variable::Identifier(identifier) => identifier_get_first_token(identifier),
        Variable::Field(field_expression) => {
            prefix_get_first_token(field_expression.mutate_prefix())
        }
        Variable::Index(index_expression) => {
            prefix_get_first_token(index_expression.mutate_prefix())
        }
    }
}

fn prefix_get_first_token(prefix: &mut Prefix) -> &mut Token {
    let mut current = prefix;
    loop {
        match current {
            Prefix::Call(call) => {
                current = call.mutate_prefix();
            }
            Prefix::Field(field_expression) => {
                current = field_expression.mutate_prefix();
            }
            Prefix::Index(index_expression) => {
                current = index_expression.mutate_prefix();
            }
            Prefix::Identifier(identifier) => break identifier_get_first_token(identifier),
            Prefix::Parenthese(parenthese_expression) => {
                break parentheses_get_first_token(parenthese_expression)
            }
        }
    }
}

fn identifier_get_first_token(identifier: &mut Identifier) -> &mut Token {
    if identifier.get_token().is_none() {
        let name = identifier.get_name().to_owned();
        identifier.set_token(Token::from_content(name));
    }
    identifier.mutate_token().unwrap()
}

fn parentheses_get_first_token(parentheses: &mut ParentheseExpression) -> &mut Token {
    if parentheses.get_tokens().is_none() {
        parentheses.set_tokens(ParentheseTokens {
            left_parenthese: Token::from_content("("),
            right_parenthese: Token::from_content(")"),
        });
    }
    &mut parentheses.mutate_tokens().unwrap().left_parenthese
}

impl RuleConfiguration for AppendTextComment {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_required_any_properties(&properties, &["text", "file"])?;
        verify_property_collisions(&properties, &["text", "file"])?;

        for (key, value) in properties {
            match key.as_str() {
                "text" => {
                    self.text_content = TextContent::Value(value.expect_string(&key)?);
                }
                "file" => {
                    self.text_content =
                        TextContent::FilePath(PathBuf::from(value.expect_string(&key)?));
                }
                "location" => {
                    self.location = match value.expect_string(&key)?.as_str() {
                        "start" => AppendLocation::Start,
                        "end" => AppendLocation::End,
                        unexpected => {
                            return Err(RuleConfigurationError::UnexpectedValue {
                                property: "location".to_owned(),
                                message: format!(
                                    "invalid value `{}` (must be `start` or `end`)",
                                    unexpected
                                ),
                            })
                        }
                    };
                }
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        APPEND_TEXT_COMMENT_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        let mut properties = RuleProperties::new();

        match self.location {
            AppendLocation::Start => {}
            AppendLocation::End => {
                properties.insert("location".to_owned(), "end".into());
            }
        }

        match &self.text_content {
            TextContent::None => {}
            TextContent::Value(value) => {
                properties.insert("text".to_owned(), value.into());
            }
            TextContent::FilePath(file_path) => {
                properties.insert(
                    "file".to_owned(),
                    file_path.to_string_lossy().to_string().into(),
                );
            }
        }

        properties
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TextContent {
    None,
    Value(String),
    FilePath(PathBuf),
}

impl Default for TextContent {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AppendLocation {
    Start,
    End,
}

impl AppendLocation {
    fn write_to_block(&self, block: &mut Block, comment: String) {
        if let Some(tokens) = block.mutate_tokens() {
            let final_token = tokens
                .final_token
                .get_or_insert_with(|| Token::from_content(""));
            self.append_comment(final_token, comment);
        } else {
            let mut token = Token::from_content("");
            self.append_comment(&mut token, comment);

            block.set_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
                final_token: Some(token),
            });
        }
    }

    fn append_comment(&self, token: &mut Token, comment: String) {
        match self {
            AppendLocation::Start => {
                token.push_leading_trivia(TriviaKind::Comment.with_content(comment));
            }
            AppendLocation::End => {
                token.push_trailing_trivia(TriviaKind::Comment.with_content(comment));
            }
        }
    }
}

impl Default for AppendLocation {
    fn default() -> Self {
        Self::Start
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    #[test]
    fn serialize_rule_with_text() {
        let rule: Box<dyn Rule> = Box::new(AppendTextComment::new("content"));

        assert_json_snapshot!("append_text_comment_with_text", rule);
    }

    #[test]
    fn serialize_rule_with_text_at_end() {
        let rule: Box<dyn Rule> = Box::new(AppendTextComment::new("content").at_end());

        assert_json_snapshot!("append_text_comment_with_text_at_end", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'append_text_comment',
            text: '',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
