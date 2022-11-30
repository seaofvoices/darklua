use std::fmt;

use full_moon::ast::Ast;

use crate::{
    ast_converter::{AstConverter, ConvertError},
    nodes::*,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parser {
    hold_token_data: bool,
}

impl Parser {
    pub fn parse(&self, code: &str) -> Result<Block, ParserError> {
        full_moon::parse(code)
            .map_err(ParserError::parsing)
            .and_then(|ast| self.convert_ast(ast).map_err(ParserError::converting))
    }

    pub fn preserve_tokens(mut self) -> Self {
        self.hold_token_data = true;
        self
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_ast(&self, ast: Ast) -> Result<Block, ConvertError> {
        AstConverter::new(self.hold_token_data).convert(ast.nodes())
    }
}

#[derive(Clone, Debug)]
enum ParserErrorKind {
    Parsing(full_moon::Error),
    Converting(ConvertError),
}

#[derive(Clone, Debug)]
pub struct ParserError {
    kind: Box<ParserErrorKind>,
}

impl ParserError {
    fn parsing(err: full_moon::Error) -> Self {
        Self {
            kind: ParserErrorKind::Parsing(err).into(),
        }
    }

    fn converting(err: ConvertError) -> Self {
        Self {
            kind: ParserErrorKind::Converting(err).into(),
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.kind {
            ParserErrorKind::Parsing(err) => write!(f, "{}", err),
            ParserErrorKind::Converting(err) => write!(f, "{}", err),
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::nodes::ReturnStatement;

    use super::*;

    macro_rules! test_parse {
        ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let parser = Parser::default();
                    let block = parser.parse($input)
                        .expect(&format!("failed to parse `{}`", $input));

                    let expect_block = $value.into();
                    pretty_assertions::assert_eq!(block, expect_block);
                }
            )*
        };
    }

    test_parse!(
        empty_string("") => Block::default(),
        empty_do("do end") => DoStatement::default(),
        empty_do_nested("do do end end") => DoStatement::new(DoStatement::default().into()),
        two_nested_empty_do_in_do_statement("do do end do end end") => DoStatement::new(
            Block::default().with_statement(DoStatement::default()).with_statement(DoStatement::default())
        ),
        triple_nested_do_statements("do do end do do end end end") => DoStatement::new(
            Block::default()
                .with_statement(DoStatement::default())
                .with_statement(DoStatement::new(DoStatement::default().into()))
        ),
        do_return_end("do return end") => DoStatement::new(ReturnStatement::default().into()),
        break_statement("break") => LastStatement::new_break(),
        return_no_values("return") => ReturnStatement::default(),
        return_true("return true") => ReturnStatement::one(Expression::from(true)),
        return_false("return false") => ReturnStatement::one(false),
        return_nil("return nil") => ReturnStatement::one(Expression::nil()),
        return_variable_arguments("return ...") => ReturnStatement::one(Expression::variable_arguments()),
        return_variable("return var") => ReturnStatement::one(Expression::identifier("var")),
        return_parentheses_true("return (true)") => ReturnStatement::one(
            Expression::from(true).in_parentheses(),
        ),
        return_true_false("return true, false") => ReturnStatement::one(Expression::from(true))
            .with_expression(false),
        empty_while_true_do("while true do end") => WhileStatement::new(Block::default(), true),
        while_false_do_break("while false do break end") => WhileStatement::new(
            LastStatement::new_break(),
            false,
        ),
        empty_repeat("repeat until true") => RepeatStatement::new(Block::default(), true),
        repeat_break("repeat break until true") => RepeatStatement::new(
            LastStatement::new_break(),
            true,
        ),
        repeat_continue("repeat continue until true") => RepeatStatement::new(
            LastStatement::new_continue(),
            true,
        ),
        local_assignment_with_no_values("local var") => LocalAssignStatement::from_variable("var"),
        multiple_local_assignment_with_no_values("local foo, bar") => LocalAssignStatement::from_variable("foo")
            .with_variable("bar"),
        local_assignment_with_one_value("local var = true") => LocalAssignStatement::from_variable("var")
            .with_value(true),
        multiple_local_assignment_with_two_values("local foo, bar = true, false") => LocalAssignStatement::from_variable("foo")
            .with_variable("bar")
            .with_value(true)
            .with_value(false),
        return_binary_and("return true and false") => ReturnStatement::one(
            BinaryExpression::new(BinaryOperator::And, true, false),
        ),
        return_zero("return 0") => ReturnStatement::one(
            NumberExpression::from_str("0").unwrap(),
        ),
        return_one("return 1") => ReturnStatement::one(
            NumberExpression::from_str("1").unwrap(),
        ),
        return_float("return 1.5") => ReturnStatement::one(
            NumberExpression::from_str("1.5").unwrap(),
        ),
        return_zero_point_five("return .5") => ReturnStatement::one(
            NumberExpression::from_str(".5").unwrap(),
        ),
        return_not_true("return not true") => ReturnStatement::one(
            UnaryExpression::new(UnaryOperator::Not, true),
        ),
        return_variable_length("return #array") => ReturnStatement::one(
            UnaryExpression::new(
                UnaryOperator::Length,
                Expression::identifier("array"),
            ),
        ),
        return_minus_variable("return -num") => ReturnStatement::one(
            UnaryExpression::new(
                UnaryOperator::Minus,
                Expression::identifier("num"),
            ),
        ),
        call_function("call()") => FunctionCall::from_name("call"),
        call_indexed_table("foo.bar()") => FunctionCall::from_prefix(
            FieldExpression::new(Prefix::from_name("foo"), "bar")
        ),
        call_method("foo:bar()") => FunctionCall::from_name("foo").with_method("bar"),
        call_method_with_one_argument("foo:bar(true)") => FunctionCall::from_name("foo")
            .with_method("bar")
            .with_argument(true),
        call_function_with_one_argument("call(true)") => FunctionCall::from_name("call")
            .with_argument(true),
        call_function_with_two_arguments("call(true, false)") => FunctionCall::from_name("call")
            .with_argument(true)
            .with_argument(false),
        call_chain_empty("call()()") => FunctionCall::from_prefix(
            FunctionCall::from_name("call")
        ),
        call_chain_with_args("call(true)(false)") => FunctionCall::from_prefix(
            FunctionCall::from_name("call").with_argument(true)
        ).with_argument(false),
        call_method_chain_empty("call():method()") => FunctionCall::from_prefix(
            FunctionCall::from_name("call")
        ).with_method("method"),
        call_method_chain_with_arguments("call(true):method(false)") => FunctionCall::from_prefix(
            FunctionCall::from_name("call").with_argument(true)
        ).with_method("method").with_argument(false),
        call_index_chain_empty("call().method()") => FunctionCall::from_prefix(
            FieldExpression::new(FunctionCall::from_name("call"), "method")
        ),
        call_with_empty_table_argument("call{}") => FunctionCall::from_name("call")
            .with_arguments(TableExpression::default()),
        call_with_empty_string_argument("call''") => FunctionCall::from_name("call")
            .with_arguments(StringExpression::empty()),
        return_call_function("return call()") => ReturnStatement::one(
            FunctionCall::from_name("call"),
        ),
        return_call_indexed_table("return foo.bar()") => ReturnStatement::one(
            FunctionCall::from_prefix(FieldExpression::new(Prefix::from_name("foo"), "bar")),
        ),
        return_call_method("return foo:bar()") => ReturnStatement::one(
            FunctionCall::from_name("foo").with_method("bar"),
        ),
        return_call_method_with_one_argument("return foo:bar(true)") => ReturnStatement::one(
            FunctionCall::from_name("foo").with_method("bar").with_argument(true),
        ),
        return_call_function_with_one_argument("return call(true)") => ReturnStatement::one(
            FunctionCall::from_name("call").with_argument(true),
        ),
        return_call_function_with_two_arguments("return call(true, false)") => ReturnStatement::one(
            FunctionCall::from_name("call")
                .with_argument(true)
                .with_argument(false),
        ),
        return_call_chain_empty("return call()()") => ReturnStatement::one(
            FunctionCall::from_prefix(FunctionCall::from_name("call")),
        ),
        return_call_chain_with_args("return call(true)(false)") => ReturnStatement::one(
            FunctionCall::from_prefix(
                FunctionCall::from_name("call").with_argument(true)
            ).with_argument(false),
        ),
        return_call_method_chain_empty("return call():method()") => ReturnStatement::one(
            FunctionCall::from_prefix(FunctionCall::from_name("call")).with_method("method"),
        ),
        return_call_method_chain_with_arguments("return call(true):method(false)")
            => ReturnStatement::one(
                FunctionCall::from_prefix(FunctionCall::from_name("call").with_argument(true))
                    .with_method("method")
                    .with_argument(false),
            ),
        return_call_index_chain_empty("return call().method()") => ReturnStatement::one(
            FunctionCall::from_prefix(FieldExpression::new(FunctionCall::from_name("call"), "method")),
        ),
        return_call_new_empty_function("return (function() end)()") => ReturnStatement::one(
            FunctionCall::from_prefix(
                ParentheseExpression::new(FunctionExpression::default())
            ),
        ),
        return_call_variable_argument("return (...)()") => ReturnStatement::one(
            FunctionCall::from_prefix(ParentheseExpression::new(Expression::variable_arguments())),
        ),
        return_call_variable_in_parentheses("return (var)()") => ReturnStatement::one(
            FunctionCall::from_prefix(ParentheseExpression::new(Expression::identifier("var"))),
        ),
        return_call_variable_in_double_parentheses("return ((var))()") => ReturnStatement::one(
            FunctionCall::from_prefix(
                ParentheseExpression::new(Expression::identifier("var").in_parentheses())
            ),
        ),
        return_field_expression("return math.huge") => ReturnStatement::one(
            FieldExpression::new(Prefix::from_name("math"), "huge")
        ),
        index_field_function_call("return call().result") => ReturnStatement::one(
            FieldExpression::new(FunctionCall::from_name("call"), "result"),
        ),
        return_index_expression("return value[true]") => ReturnStatement::one(
            IndexExpression::new(Prefix::from_name("value"), true)
        ),
        return_empty_table("return {}") => ReturnStatement::one(TableExpression::default()),
        return_array_with_one_element("return {true}") => ReturnStatement::one(
            TableExpression::default().append_array_value(true)
        ),
        return_array_with_two_elements("return {true, false}") => ReturnStatement::one(
            TableExpression::default()
                .append_array_value(true)
                .append_array_value(false)

        ),
        return_array_with_one_field("return { field = true }") => ReturnStatement::one(
            TableExpression::default().append_field("field", true)
        ),
        return_array_with_one_key_expression("return { [false] = true }") => ReturnStatement::one(
            TableExpression::default().append_index(false, true)
        ),
        assign_variable("var = true") => AssignStatement::from_variable(
            Variable::new("var"),
            true,
        ),
        assign_two_variables("var, var2 = true, false") => AssignStatement::from_variable(
            Variable::new("var"),
            true,
        ).append_assignment(Variable::new("var2"), false),
        assign_one_variable_with_two_values("var = 0b1010, ...") => AssignStatement::new(
            vec![Variable::new("var")],
            vec!["0b1010".parse::<NumberExpression>().unwrap().into(), Expression::variable_arguments()],
        ),
        assign_field("var.field = true") => AssignStatement::from_variable(
            FieldExpression::new(Prefix::from_name("var"), "field"),
            true,
        ),
        assign_field_and_variable("var.field, other = true, 1 + value") =>
            AssignStatement::from_variable(
                FieldExpression::new(Prefix::from_name("var"), "field"),
                true,
            ).append_assignment(
                Variable::new("other"),
                BinaryExpression::new(BinaryOperator::Plus, 1.0, Expression::identifier("value"))
            ),
        assign_index("var[false] = true") => AssignStatement::from_variable(
            IndexExpression::new(Prefix::from_name("var"), false),
            true,
        ),
        return_empty_function("return function() end") => ReturnStatement::one(
            FunctionExpression::default(),
        ),
        return_empty_function_with_one_param("return function(a) end") => ReturnStatement::one(
            FunctionExpression::default().with_parameter("a"),
        ),
        return_empty_function_with_two_params("return function(a, b) end") => ReturnStatement::one(
            FunctionExpression::default().with_parameter("a").with_parameter("b"),
        ),
        return_empty_variadic_function("return function(...) end") => ReturnStatement::one(
            FunctionExpression::default().variadic(),
        ),
        return_empty_variadic_function_with_one_param("return function(a, ...) end")
            => ReturnStatement::one(
                FunctionExpression::default().with_parameter("a").variadic(),
            ),
        return_function_that_returns("return function() return true end")
            => ReturnStatement::one(
                FunctionExpression::from_block(ReturnStatement::one(Expression::from(true)))
            ),
        empty_if_statement("if true then end") => IfStatement::create(true, Block::default()),
        if_statement_returns("if true then return end") => IfStatement::create(
            Expression::from(true),
            ReturnStatement::default(),
        ),
        empty_if_statement_with_empty_else("if true then else end")
            => IfStatement::create(true, Block::default())
                .with_else_block(Block::default()),
        empty_if_statement_with_empty_elseif("if true then elseif false then end")
            => IfStatement::create(true, Block::default())
                .with_new_branch(false, Block::default()),
        empty_if_statement_with_empty_elseif_and_empty_else("if true then elseif false then else end")
            => IfStatement::create(true, Block::default())
                .with_new_branch(false, Block::default())
                .with_else_block(Block::default()),
        empty_if_statement_with_returning_else("if true then else return end")
            => IfStatement::create(true, Block::default())
                .with_else_block(ReturnStatement::default()),
        empty_local_function("local function name() end")
            => LocalFunctionStatement::from_name("name", Block::default()),
        empty_local_function_variadic("local function name(...) end")
            => LocalFunctionStatement::from_name("name", Block::default()).variadic(),
        empty_local_function_variadic_with_one_parameter("local function name(a, ...) end")
            => LocalFunctionStatement::from_name("name", Block::default())
                .with_parameter("a")
                .variadic(),
        local_function_return("local function name() return end")
            => LocalFunctionStatement::from_name("name", ReturnStatement::default()),

        empty_function_statement("function name() end")
            => FunctionStatement::from_name("name", Block::default()),
        empty_function_statement_variadic("function name(...) end")
            => FunctionStatement::from_name("name", Block::default()).variadic(),
        empty_function_statement_variadic_with_one_parameter("function name(a, ...) end")
            => FunctionStatement::from_name("name", Block::default())
                .with_parameter("a")
                .variadic(),
        function_statement_return("function name() return end")
            => FunctionStatement::from_name("name", ReturnStatement::default()),
        empty_generic_for("for key in pairs(t) do end") => GenericForStatement::new(
            vec!["key".into()],
            vec![
                FunctionCall::from_name("pairs")
                    .with_argument(Expression::identifier("t"))
                    .into(),
            ],
            Block::default(),
        ),
        empty_generic_for_multiple_variables("for key, value in pairs(t) do end") => GenericForStatement::new(
            vec!["key".into(), "value".into()],
            vec![
                FunctionCall::from_name("pairs")
                    .with_argument(Expression::identifier("t"))
                    .into(),
            ],
            Block::default(),
        ),
        empty_generic_for_multiple_values("for key in next, t do end") => GenericForStatement::new(
            vec!["key".into()],
            vec![Expression::identifier("next"), Expression::identifier("t")],
            Block::default(),
        ),
        generic_for_break("for key in pairs(t) do break end") => GenericForStatement::new(
            vec!["key".into()],
            vec![
                FunctionCall::from_name("pairs")
                    .with_argument(Expression::identifier("t"))
                    .into(),
            ],
            LastStatement::new_break(),
        ),
        empty_numeric_for("for i=start, bound do end") => NumericForStatement::new(
            "i",
            Expression::identifier("start"),
            Expression::identifier("bound"),
            None,
            Block::default(),
        ),
        empty_numeric_for_with_step("for i=start, bound, step do end") => NumericForStatement::new(
            "i",
            Expression::identifier("start"),
            Expression::identifier("bound"),
            Some(Expression::identifier("step")),
            Block::default(),
        ),
        numeric_for_that_breaks("for i=start, bound do break end") => NumericForStatement::new(
            "i",
            Expression::identifier("start"),
            Expression::identifier("bound"),
            None,
            LastStatement::new_break(),
        ),
        compound_increment("var += amount") => CompoundAssignStatement::new(
            CompoundOperator::Plus,
            Variable::new("var"),
            Expression::identifier("amount"),
        ),
    );

    mod parse_with_tokens {
        use super::*;

        macro_rules! test_parse_block_with_tokens {
            ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let parser = Parser::default().preserve_tokens();
                        let block = parser.parse($input)
                            .expect(&format!("failed to parse `{}`", $input));

                        let expect_block = $value;

                        pretty_assertions::assert_eq!(block, expect_block);
                    }
                )*
            };
        }

        macro_rules! test_parse_statement_with_tokens {
            ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
                test_parse_block_with_tokens!(
                    $(
                        $name($input) => Block::from($value).with_tokens(BlockTokens {
                            semicolons: vec![None],
                            last_semicolon: None,
                        }),
                    )*
                );
            };
        }

        macro_rules! test_parse_last_statement_with_tokens {
            ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
                test_parse_block_with_tokens!(
                    $(
                        $name($input) => Block::from($value).with_tokens(BlockTokens {
                            semicolons: Vec::new(),
                            last_semicolon: None,
                        }),
                    )*
                );
            };
        }

        fn create_true(start: usize, whitespace_length: usize) -> Expression {
            let end = start + 4;
            let token = Token::new_with_line(start, end, 1);
            Expression::True(Some(if whitespace_length == 0 {
                token
            } else {
                token.with_trailing_trivia(TriviaKind::Whitespace.at(
                    end,
                    end + whitespace_length,
                    1,
                ))
            }))
        }

        fn create_identifier(
            identifier: &str,
            start: usize,
            whitespace_length: usize,
        ) -> Identifier {
            create_identifier_at_line(identifier, start, whitespace_length, 1)
        }

        fn create_identifier_at_line(
            identifier: &str,
            start: usize,
            whitespace_length: usize,
            line: usize,
        ) -> Identifier {
            let end = start + identifier.len();
            Identifier::new(identifier).with_token({
                let token = Token::new_with_line(start, end, line);
                if whitespace_length != 0 {
                    token.with_trailing_trivia(TriviaKind::Whitespace.at(
                        end,
                        end + whitespace_length,
                        line,
                    ))
                } else {
                    token
                }
            })
        }

        fn spaced_token(start: usize, end: usize) -> Token {
            spaced_token_at_line(start, end, 1)
        }

        fn spaced_token_at_line(start: usize, end: usize, line: usize) -> Token {
            Token::new_with_line(start, end, line).with_trailing_trivia(TriviaKind::Whitespace.at(
                end,
                end + 1,
                line,
            ))
        }

        fn default_block() -> Block {
            Block::default().with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
            })
        }

        fn token_at_first_line(start: usize, end: usize) -> Token {
            Token::new_with_line(start, end, 1)
        }

        test_parse_last_statement_with_tokens!(
            return_with_comment("return -- comment") => ReturnStatement::default()
                .with_tokens(ReturnTokens {
                    r#return: token_at_first_line(0, 6)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(6, 7, 1))
                        .with_trailing_trivia(TriviaKind::Comment.at(7, 17, 1)),
                    commas: Vec::new(),
                }),
            return_true("return true") => ReturnStatement::one(create_true(7, 0))
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_false("return false") => ReturnStatement::one(
                Expression::False(Some(token_at_first_line(7, 12)))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_nil("return nil") => ReturnStatement::one(
                Expression::Nil(Some(token_at_first_line(7, 10)))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_variable_arguments("return ...") => ReturnStatement::one(
                Expression::VariableArguments(Some(token_at_first_line(7, 10)))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_single_quote_string("return ''") => ReturnStatement::one(
                StringExpression::empty().with_token(token_at_first_line(7, 9))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_double_quote_string("return \"\"") => ReturnStatement::one(
                StringExpression::empty().with_token(token_at_first_line(7, 9))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_double_quote_string("return \"abc\"") => ReturnStatement::one(
                StringExpression::from_value("abc").with_token(token_at_first_line(7, 12))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_integer_number("return 123") => ReturnStatement::one(
                DecimalNumber::new(123.0).with_token(token_at_first_line(7, 10))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_float("return 12.34 -- value") => ReturnStatement::one(
                DecimalNumber::new(12.34).with_token(
                    spaced_token(7, 12).with_trailing_trivia(TriviaKind::Comment.at(13, 21, 1))
                )
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_binary_number("return 0b1010") => ReturnStatement::one(
                BinaryNumber::new(0b1010, false).with_token(token_at_first_line(7, 13))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_hexadecimal_number("return 0x12EF") => ReturnStatement::one(
                HexNumber::new(0x12EF, false).with_token(token_at_first_line(7, 13))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_table("return {--[[ inside ]]}") => ReturnStatement::one(
                TableExpression::default().with_tokens(TableTokens {
                    opening_brace: token_at_first_line(7, 8)
                        .with_trailing_trivia(TriviaKind::Comment.at(8, 22, 1)),
                    closing_brace: token_at_first_line(22, 23),
                    separators: Vec::new(),
                })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_one_element("return { true} ") => ReturnStatement::one(
                TableExpression::default()
                    .append_array_value(create_true(9, 0))
                    .with_tokens(TableTokens {
                        opening_brace: spaced_token(7, 8),
                        closing_brace: spaced_token(13, 14),
                        separators: Vec::new(),
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_two_elements("return {true, true}") => ReturnStatement::one(
                TableExpression::default()
                    .append_array_value(create_true(8, 0))
                    .append_array_value(create_true(14, 0))
                    .with_tokens(TableTokens {
                        opening_brace: token_at_first_line(7, 8),
                        closing_brace: token_at_first_line(18, 19),
                        separators: vec![spaced_token(12, 13)],
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_one_field("return { field = true; }") => ReturnStatement::one(
                TableExpression::default()
                    .append_entry(
                        TableFieldEntry::new(
                            create_identifier("field", 9, 1),
                            create_true(17, 0),
                        ).with_token(spaced_token(15, 16))
                    )
                    .with_tokens(TableTokens {
                        opening_brace: spaced_token(7, 8),
                        closing_brace: token_at_first_line(23, 24),
                        separators: vec![spaced_token(21, 22)],
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_one_key_expression("return { [var] = true }") => ReturnStatement::one(
                TableExpression::default()
                    .append_entry(
                        TableIndexEntry::new(
                            create_identifier("var", 10, 0),
                            create_true(17, 1),
                        ).with_tokens(TableIndexEntryTokens {
                            opening_bracket: token_at_first_line(9, 10),
                            closing_bracket: spaced_token(13, 14),
                            equal: spaced_token(15, 16),
                        })
                    )
                    .with_tokens(TableTokens {
                        opening_brace: spaced_token(7, 8),
                        closing_brace: token_at_first_line(22, 23),
                        separators: Vec::new(),
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_field_expression("return math.huge") => ReturnStatement::one(
                FieldExpression::new(
                    Prefix::from_name(create_identifier("math", 7, 0)),
                    create_identifier("huge", 12, 0)
                ).with_token(token_at_first_line(11, 12))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_double_field_expression("return table.ok .result") => ReturnStatement::one(
                FieldExpression::new(
                    FieldExpression::new(
                        Prefix::from_name(create_identifier("table", 7, 0)),
                        create_identifier("ok", 13, 1)
                    ).with_token(token_at_first_line(12, 13)),
                    create_identifier("result", 17, 0)
                ).with_token(token_at_first_line(16, 17))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_index_expression("return value [ true ] ") => ReturnStatement::one(
                IndexExpression::new(
                    create_identifier("value", 7, 1),
                    create_true(15, 1)
                ).with_tokens(IndexExpressionTokens {
                    opening_bracket: spaced_token(13, 14),
                    closing_bracket: spaced_token(20, 21),
                })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_true_and_true("return true and true") => ReturnStatement::default()
                .with_expression(
                    BinaryExpression::new(
                        BinaryOperator::And,
                        create_true(7, 1),
                        create_true(16, 0),
                    ).with_token(spaced_token(12, 15))
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_not_true("return not true") => ReturnStatement::default()
                .with_expression(
                    UnaryExpression::new(
                        UnaryOperator::Not,
                        create_true(11, 0),
                    ).with_token(spaced_token(7, 10))
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_parenthese_expression("return ( true )") => ReturnStatement::default()
                .with_expression(
                    ParentheseExpression::new(create_true(9, 1))
                        .with_tokens(
                            ParentheseTokens {
                                left_parenthese: spaced_token(7, 8),
                                right_parenthese: token_at_first_line(14, 15),
                            }
                        )
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_empty_function("return function  ( --[[params]]) end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block())
                    .with_tokens(FunctionExpressionTokens {
                        function: token_at_first_line(7, 15)
                            .with_trailing_trivia(TriviaKind::Whitespace.at(15, 17, 1)),
                        opening_parenthese: token_at_first_line(17, 18)
                            .with_trailing_trivia(TriviaKind::Whitespace.at(18, 19, 1))
                            .with_trailing_trivia(TriviaKind::Comment.at(19, 31, 1)),
                        closing_parenthese: spaced_token(31, 32),
                        end: token_at_first_line(33, 36),
                        parameter_commas: Vec::new(),
                        variable_arguments: None,
                    }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_function_with_one_param("return function(a )end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block()).with_parameter(
                    create_identifier("a", 16, 1)
                ).with_tokens(FunctionExpressionTokens {
                    function: token_at_first_line(7, 15),
                    opening_parenthese: token_at_first_line(15, 16),
                    closing_parenthese: token_at_first_line(18, 19),
                    end: token_at_first_line(19, 22),
                    parameter_commas: Vec::new(),
                    variable_arguments: None,
                }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_function_with_two_params("return function(a, b--[[foo]]) end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block())
                    .with_parameter(Identifier::new("a").with_token(token_at_first_line(16, 17)))
                    .with_parameter(
                        Identifier::new("b").with_token(
                            token_at_first_line(19, 20).with_trailing_trivia(TriviaKind::Comment.at(20, 29, 1))
                        )
                    )
                    .with_tokens(FunctionExpressionTokens {
                        function: token_at_first_line(7, 15),
                        opening_parenthese: token_at_first_line(15, 16),
                        closing_parenthese: spaced_token(29, 30),
                        end: token_at_first_line(31, 34),
                        parameter_commas: vec![spaced_token(17, 18)],
                        variable_arguments: None,
                    }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_variadic_function("return function(... ) end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block())
                    .variadic()
                    .with_tokens(FunctionExpressionTokens {
                        function: token_at_first_line(7, 15),
                        opening_parenthese: token_at_first_line(15, 16),
                        closing_parenthese: spaced_token(20, 21),
                        end: token_at_first_line(22, 25),
                        parameter_commas: Vec::new(),
                        variable_arguments: Some(spaced_token(16, 19)),
                    }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_two_values("return true ,  true--end") => ReturnStatement::default()
                .with_expression(create_true(7, 1))
                .with_expression(Expression::True(Some(
                    token_at_first_line(15, 19).with_trailing_trivia(TriviaKind::Comment.at(19, 24, 1))
                )))
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: vec![
                        token_at_first_line(12, 13).with_trailing_trivia(TriviaKind::Whitespace.at(13, 15, 1))
                    ],
                }),
            return_variable("return var") => ReturnStatement::default()
                .with_expression(
                    Identifier::new("var").with_token(token_at_first_line(7, 10))
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            break_statement("break") => LastStatement::Break(Some(token_at_first_line(0, 5))),
            break_statement_with_comment("break-- bye") => LastStatement::Break(Some(
                token_at_first_line(0, 5).with_trailing_trivia(TriviaKind::Comment.at(5, 11, 1))
            )),
            continue_statement("continue") => LastStatement::Continue(Some(token_at_first_line(0, 8))),
            continue_statement_with_comment("continue-- bye") => LastStatement::Continue(Some(
                token_at_first_line(0, 8).with_trailing_trivia(TriviaKind::Comment.at(8, 14, 1))
            )),
        );

        test_parse_statement_with_tokens!(
            empty_local_function("local function name ()end") => LocalFunctionStatement::from_name(
                create_identifier("name", 15, 1),
                default_block()
            ).with_tokens(LocalFunctionTokens {
                local: spaced_token(0, 5),
                function: spaced_token(6, 14),
                opening_parenthese: token_at_first_line(20, 21),
                closing_parenthese: token_at_first_line(21, 22),
                end: token_at_first_line(22, 25),
                parameter_commas: Vec::new(),
                variable_arguments: None,
            }),
            empty_local_function_variadic("local function name(...)end") => LocalFunctionStatement::from_name(
                Identifier::new("name").with_token(token_at_first_line(15, 19)),
                default_block(),
            )
            .variadic()
            .with_tokens(LocalFunctionTokens {
                local: spaced_token(0, 5),
                function: spaced_token(6, 14),
                opening_parenthese: token_at_first_line(19, 20),
                closing_parenthese: token_at_first_line(23, 24),
                end: token_at_first_line(24, 27),
                parameter_commas: Vec::new(),
                variable_arguments: Some(token_at_first_line(20, 23)),
            }),
            empty_local_function_variadic_with_one_parameter("local function name(a,b) end")
                => LocalFunctionStatement::from_name(
                    Identifier::new("name").with_token(token_at_first_line(15, 19)),
                    default_block(),
                )
                .with_parameter(Identifier::new("a").with_token(token_at_first_line(20, 21)))
                .with_parameter(Identifier::new("b").with_token(token_at_first_line(22, 23)))
                .with_tokens(LocalFunctionTokens {
                    local: spaced_token(0, 5),
                    function: spaced_token(6, 14),
                    opening_parenthese: token_at_first_line(19, 20),
                    closing_parenthese: spaced_token(23, 24),
                    end: token_at_first_line(25, 28),
                    parameter_commas: vec![token_at_first_line(21, 22)],
                    variable_arguments: None,
                }),
            call_function("call()") => FunctionCall::from_name(
                create_identifier("call", 0, 0)
            ).with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                opening_parenthese: token_at_first_line(4, 5),
                closing_parenthese: token_at_first_line(5, 6),
                commas: Vec::new(),
            })).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_indexed_table("foo.bar()") => FunctionCall::from_prefix(
                FieldExpression::new(
                    create_identifier("foo", 0, 0),
                    create_identifier("bar", 4, 0)
                ).with_token(token_at_first_line(3, 4))
            ).with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                opening_parenthese: token_at_first_line(7, 8),
                closing_parenthese: token_at_first_line(8, 9),
                commas: Vec::new(),
            })).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_method("foo: bar()") => FunctionCall::from_name(create_identifier("foo", 0, 0))
                .with_method(create_identifier("bar", 5, 0))
                .with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                    opening_parenthese: token_at_first_line(8, 9),
                    closing_parenthese: token_at_first_line(9, 10),
                    commas: Vec::new(),
                }))
                .with_tokens(FunctionCallTokens {
                    colon: Some(spaced_token(3, 4)),
                }),
            call_method_with_one_argument("foo:bar( true )") => FunctionCall::from_name(
                create_identifier("foo", 0, 0)
            )
            .with_method(create_identifier("bar", 4, 0))
            .with_arguments(
                TupleArguments::default()
                    .with_argument(create_true(9, 1))
                    .with_tokens(TupleArgumentsTokens {
                        opening_parenthese: spaced_token(7, 8),
                        closing_parenthese: token_at_first_line(14, 15),
                        commas: Vec::new(),
                    })
                )
            .with_tokens(FunctionCallTokens {
                colon: Some(token_at_first_line(3, 4)),
            }),
            call_function_with_one_argument("call ( true ) ") =>  FunctionCall::from_name(
                create_identifier("call", 0, 1)
            )
            .with_arguments(
                TupleArguments::default()
                    .with_argument(create_true(7, 1))
                    .with_tokens(TupleArgumentsTokens {
                        opening_parenthese: spaced_token(5, 6),
                        closing_parenthese: spaced_token(12, 13),
                        commas: Vec::new(),
                    })
                )
            .with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_function_with_two_arguments("call(true, true)") =>  FunctionCall::from_name(
                create_identifier("call", 0, 0)
            )
            .with_arguments(
                TupleArguments::default()
                    .with_argument(create_true(5, 0))
                    .with_argument(create_true(11, 0))
                    .with_tokens(TupleArgumentsTokens {
                        opening_parenthese: token_at_first_line(4, 5),
                        closing_parenthese: token_at_first_line(15, 16),
                        commas: vec![spaced_token(9, 10)],
                    })
                )
            .with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_chain_with_args("call(true)( )") => FunctionCall::from_prefix(
                FunctionCall::from_name(create_identifier("call", 0, 0))
                    .with_arguments(
                        TupleArguments::default()
                            .with_argument(create_true(5, 0))
                            .with_tokens(TupleArgumentsTokens {
                                opening_parenthese: token_at_first_line(4, 5),
                                closing_parenthese: token_at_first_line(9, 10),
                                commas: Vec::new(),
                            })
                        )
                    .with_tokens(FunctionCallTokens {
                        colon: None,
                    }),
            )
            .with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                opening_parenthese: spaced_token(10, 11),
                closing_parenthese: token_at_first_line(12, 13),
                commas: Vec::new(),
            }))
            .with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_with_empty_table_argument("call{ }") => FunctionCall::from_name(
                create_identifier("call", 0, 0)
            ).with_arguments(TableExpression::default().with_tokens(TableTokens {
                opening_brace: spaced_token(4, 5),
                closing_brace: token_at_first_line(6, 7),
                separators: Vec::new(),
            })).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_with_empty_string_argument("call ''") => FunctionCall::from_name(
                create_identifier("call", 0, 1)
            ).with_arguments(
                StringExpression::empty().with_token(token_at_first_line(5, 7))
            ).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            empty_do("do end") => DoStatement::new(default_block())
                .with_tokens(DoTokens {
                    r#do: spaced_token(0, 2),
                    end: token_at_first_line(3, 6),
                }),
            empty_do_with_long_comment("do --[[ hello ]] end") => DoStatement::new(default_block())
                .with_tokens(DoTokens {
                    r#do: token_at_first_line(0, 2)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(2, 3, 1))
                        .with_trailing_trivia(TriviaKind::Comment.at(3, 16, 1))
                        .with_trailing_trivia(TriviaKind::Whitespace.at(16, 17, 1)),
                    end: token_at_first_line(17, 20),
                }),
            assign_variable("var = true") => AssignStatement::from_variable(
                create_identifier("var", 0, 1),
                Expression::True(Some(token_at_first_line(6, 10))),
            ).with_tokens(AssignTokens {
                equal: spaced_token(4, 5),
                variable_commas: Vec::new(),
                value_commas: Vec::new(),
            }),
            assign_two_variables_with_two_values("var, var2 = true, true") => AssignStatement::from_variable(
                Identifier::new("var").with_token(token_at_first_line(0, 3)),
                create_true(12, 0),
            ).append_assignment(
                create_identifier("var2", 5, 1),
                Expression::True(Some(token_at_first_line(18, 22))),
            ).with_tokens(AssignTokens {
                equal: spaced_token(10, 11),
                variable_commas: vec![spaced_token(3, 4)],
                value_commas: vec![spaced_token(16, 17)],
            }),
            empty_function_statement("function name() end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_tokens(FunctionNameTokens { periods: Vec::new(), colon: None }),
                    default_block(),
                    Vec::new(),
                    false,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(13, 14),
                    closing_parenthese: spaced_token(14, 15),
                    end: token_at_first_line(16, 19),
                    parameter_commas: Vec::new(),
                    variable_arguments: None
                }),
            empty_function_statement_with_field("function name.field ()end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_field(
                        Identifier::new("field").with_token(spaced_token(14, 19))
                    ).with_tokens(FunctionNameTokens {
                        periods: vec![token_at_first_line(13, 14)],
                        colon: None,
                    }),
                    default_block(),
                    Vec::new(),
                    false,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(20, 21),
                    closing_parenthese: token_at_first_line(21, 22),
                    end: token_at_first_line(22, 25),
                    parameter_commas: Vec::new(),
                    variable_arguments: None
                }),
            empty_function_statement_with_method("function name:method ()end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    )
                    .with_method(create_identifier("method", 14, 1))
                    .with_tokens(FunctionNameTokens {
                        periods: Vec::new(),
                        colon: Some(token_at_first_line(13, 14)),
                    }),
                    default_block(),
                    Vec::new(),
                    false,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(21, 22),
                    closing_parenthese: token_at_first_line(22, 23),
                    end: token_at_first_line(23, 26),
                    parameter_commas: Vec::new(),
                    variable_arguments: None
                }),
            empty_function_statement_variadic("function name(...) end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_tokens(FunctionNameTokens { periods: Vec::new(), colon: None }),
                    default_block(),
                    Vec::new(),
                    true,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(13, 14),
                    closing_parenthese: token_at_first_line(17, 18)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(18, 19, 1)),
                    end: token_at_first_line(19, 22),
                    parameter_commas: Vec::new(),
                    variable_arguments: Some(token_at_first_line(14, 17))
                }),
            empty_function_statement_variadic_with_one_parameter("function name(a,...)end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_tokens(FunctionNameTokens { periods: Vec::new(), colon: None }),
                    default_block(),
                    vec![
                        Identifier::new("a").with_token(token_at_first_line(14, 15))
                    ],
                    true,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(13, 14),
                    closing_parenthese: token_at_first_line(19, 20),
                    end: token_at_first_line(20, 23),
                    parameter_commas: vec![
                        token_at_first_line(15, 16),
                    ],
                    variable_arguments: Some(token_at_first_line(16, 19))
                }),
            empty_generic_for("for key in foo do end") => GenericForStatement::new(
                vec![
                    create_identifier("key", 4, 1),
                ],
                vec![
                    create_identifier("foo", 11, 1).into(),
                ],
                default_block(),
            ).with_tokens(GenericForTokens {
                r#for: spaced_token(0, 3),
                r#in: spaced_token(8, 10),
                r#do: spaced_token(15, 17),
                end: token_at_first_line(18, 21),
                identifier_commas: Vec::new(),
                value_commas: Vec::new(),
            }),
            empty_generic_for_multiple_variables("for key, value in foo do end") => GenericForStatement::new(
                vec![
                    Identifier::new("key").with_token(token_at_first_line(4, 7)),
                    create_identifier("value", 9, 1),
                ],
                vec![
                    create_identifier("foo", 18, 1).into(),
                ],
                default_block(),
            ).with_tokens(GenericForTokens {
                r#for: spaced_token(0, 3),
                r#in: spaced_token(15, 17),
                r#do: spaced_token(22, 24),
                end: token_at_first_line(25, 28),
                identifier_commas: vec![spaced_token(7, 8)],
                value_commas: Vec::new(),
            }),
            empty_generic_for_multiple_values("for key in next , t do end") => GenericForStatement::new(
                vec![create_identifier("key", 4, 1)],
                vec![
                    create_identifier("next", 11, 1).into(),
                    create_identifier("t", 18, 1).into(),
                ],
                default_block(),
            ).with_tokens(GenericForTokens {
                r#for: spaced_token(0, 3),
                r#in: spaced_token(8, 10),
                r#do: spaced_token(20, 22),
                end: token_at_first_line(23, 26),
                identifier_commas: Vec::new(),
                value_commas: vec![
                    token_at_first_line(16, 17).with_trailing_trivia(TriviaKind::Whitespace.at(17, 18, 1)),
                ],
            }),
            empty_if_statement("if true then end") => IfStatement::create(
                create_true(3, 1),
                default_block()
            ).with_tokens(IfStatementTokens {
                r#if: token_at_first_line(0, 2).with_trailing_trivia(TriviaKind::Whitespace.at(2, 3, 1)),
                then: token_at_first_line(8, 12).with_trailing_trivia(TriviaKind::Whitespace.at(12, 13, 1)),
                end: token_at_first_line(13, 16),
                r#else: None,
            }),
            empty_if_statement_with_empty_else("if true then else end") => IfStatement::create(
                create_true(3, 1),
                default_block()
            )
            .with_else_block(default_block())
            .with_tokens(IfStatementTokens {
                r#if: spaced_token(0, 2),
                then: spaced_token(8, 12),
                end: token_at_first_line(18, 21),
                r#else: Some(spaced_token(13, 17)),
            }),
            empty_if_statement_with_empty_elseif("if true then elseif true then end")
                => IfStatement::create(create_true(3, 1), default_block())
                .with_branch(
                    IfBranch::new(create_true(20, 1), default_block())
                        .with_tokens(IfBranchTokens {
                            elseif: spaced_token(13, 19),
                            then: spaced_token(25, 29),
                        })
                )
                .with_tokens(IfStatementTokens {
                    r#if: spaced_token(0, 2),
                    then: spaced_token(8, 12),
                    end: token_at_first_line(30, 33),
                    r#else: None,
                }),
            local_assignment_with_no_values("local var ") => LocalAssignStatement::from_variable(
                create_identifier("var", 6, 1),
            ).with_tokens(LocalAssignTokens {
                local: spaced_token(0, 5),
                equal: None,
                variable_commas: Vec::new(),
                value_commas: Vec::new(),
             }),
            multiple_local_assignment_with_no_values("local foo, bar") => LocalAssignStatement::from_variable(
                create_identifier("foo", 6, 0)
            )
            .with_variable(create_identifier("bar", 11, 0))
            .with_tokens(LocalAssignTokens {
                local: spaced_token(0, 5),
                equal: None,
                variable_commas: vec![spaced_token(9, 10)],
                value_commas: Vec::new(),
             }),
            multiple_local_assignment_with_two_values("local foo, bar = true, true")
                => LocalAssignStatement::from_variable(
                    create_identifier("foo", 6, 0)
                )
                .with_variable(create_identifier("bar", 11, 1))
                .with_value(create_true(17, 0))
                .with_value(create_true(23, 0))
                .with_tokens(LocalAssignTokens {
                    local: spaced_token(0, 5),
                    equal: Some(spaced_token(15, 16)),
                    variable_commas: vec![spaced_token(9, 10)],
                    value_commas: vec![spaced_token(21, 22)],
                 }),

            empty_numeric_for("for i = start,bound do end") => NumericForStatement::new(
                create_identifier("i", 4, 1),
                create_identifier("start", 8, 0),
                create_identifier("bound", 14, 1),
                None,
                default_block(),
            ).with_tokens(NumericForTokens {
                r#for: spaced_token(0, 3),
                equal: spaced_token(6, 7),
                r#do: spaced_token(20, 22),
                end: token_at_first_line(23, 26),
                end_comma: token_at_first_line(13, 14),
                step_comma: None,
            }),
            empty_numeric_for_with_step("for i = start , bound , step do end")
                => NumericForStatement::new(
                    create_identifier("i", 4, 1),
                    create_identifier("start", 8, 1),
                    create_identifier("bound", 16, 1),
                    Some(create_identifier("step", 24, 1).into()),
                    default_block(),
                ).with_tokens(NumericForTokens {
                    r#for: spaced_token(0, 3),
                    equal: spaced_token(6, 7),
                    r#do: spaced_token(29, 31),
                    end: token_at_first_line(32, 35),
                    end_comma: spaced_token(14, 15),
                    step_comma: Some(spaced_token(22, 23)),
                }),
            empty_repeat("repeat until true") => RepeatStatement::new(
                default_block(),
                create_true(13, 0),
            ).with_tokens(RepeatTokens {
                repeat: spaced_token(0, 6),
                until: spaced_token(7, 12),
            }),
            empty_while("while true do end") => WhileStatement::new(
                default_block(),
                create_true(6, 1),
            ).with_tokens(WhileTokens {
                r#while: token_at_first_line(0, 5)
                    .with_trailing_trivia(TriviaKind::Whitespace.at(5, 6, 1)),
                r#do: token_at_first_line(11, 13)
                    .with_trailing_trivia(TriviaKind::Whitespace.at(13, 14, 1)),
                end: token_at_first_line(14, 17),
            }),
            compound_increment("var += amount") => CompoundAssignStatement::new(
                CompoundOperator::Plus,
                create_identifier("var", 0, 1),
                create_identifier("amount", 7, 0),
            ).with_tokens(CompoundAssignTokens { operator: spaced_token(4, 6) }),
        );

        test_parse_block_with_tokens!(
            return_nothing_with_semicolon("return;") => Block::from(
                ReturnStatement::default()
                    .with_tokens(ReturnTokens {
                        r#return: token_at_first_line(0, 6),
                        commas: Vec::new(),
                    }),
            ).with_tokens(BlockTokens {
                semicolons: vec![],
                last_semicolon: Some(token_at_first_line(6, 7)),
            }),
            return_nothing_with_semicolon_and_comment("return; -- return nothing") => Block::from(
                ReturnStatement::default()
                    .with_tokens(ReturnTokens {
                        r#return: token_at_first_line(0, 6),
                        commas: Vec::new(),
                    }),
            ).with_tokens(BlockTokens {
                semicolons: vec![],
                last_semicolon: Some(
                    token_at_first_line(6, 7)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(7, 8, 1))
                        .with_trailing_trivia(TriviaKind::Comment.at(8, 25, 1))
                ),
            }),
            two_local_declarations("local a;\nlocal b;\n") => Block::from(
                LocalAssignStatement::from_variable(create_identifier("a", 6, 0))
                    .with_tokens(LocalAssignTokens {
                        local: spaced_token(0, 5),
                        equal: None,
                        variable_commas: Vec::new(),
                        value_commas: Vec::new(),
                    })
            ).with_statement(
                LocalAssignStatement::from_variable(create_identifier_at_line("b", 15, 0, 2))
                    .with_tokens(LocalAssignTokens {
                        local: spaced_token_at_line(9, 14, 2),
                        equal: None,
                        variable_commas: Vec::new(),
                        value_commas: Vec::new(),
                    })
            ).with_tokens(BlockTokens {
                semicolons: vec![
                    Some(spaced_token(7, 8)),
                    Some(spaced_token_at_line(16, 17, 2)),
                ],
                last_semicolon: None,
            }),
        );
    }
}
