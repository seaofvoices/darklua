use crate::nodes::*;

use core::convert::TryInto;
use luaparser::{
    builders,
    FastParser,
    NodeTypes,
    ParsingError,
};

pub struct FastParserNodes;

impl NodeTypes for FastParserNodes {
    type Block = Block;
    type Statement = Statement;
    type LastStatement = LastStatement;

    type AssignStatement = AssignStatement;
    type DoStatement = DoStatement;
    type CallStatement = FunctionCall;
    type FunctionStatement = FunctionStatement;
    type GenericForStatement = GenericForStatement;
    type IfStatement = IfStatement;
    type LocalAssignStatement = LocalAssignStatement;
    type LocalFunctionStatement = LocalFunctionStatement;
    type NumericForStatement = NumericForStatement;
    type RepeatStatement = RepeatStatement;
    type WhileStatement = WhileStatement;

    type Expression = Expression;

    type Arguments = Arguments;
    type Prefix = Prefix;

    type Variable = Variable;
    type FieldExpression = FieldExpression;
    type IndexExpression = IndexExpression;

    type BinaryOperator = BinaryOperator;
    type BinaryExpression = BinaryExpression;
    type CallExpression = FunctionCall;
    type FunctionExpression = FunctionExpression;
    type NumberExpression = NumberExpression;
    type StringExpression = StringExpression;
    type TableEntry = TableEntry;
    type TableExpression = TableExpression;
    type UnaryOperator = UnaryOperator;
    type UnaryExpression = UnaryExpression;
}

#[derive(Default)]
struct TokenParser;

impl FastParser for TokenParser {
    type Types = FastParserNodes;
}

/// A Lua 5.1 parser the creates the abstract syntax tree.
#[derive(Default)]
pub struct Parser {
    parser: TokenParser,
}

impl Parser {
    pub fn parse<'a>(&self, input: &'a str) -> Result<Block, ParsingError> {
        self.parser.parse(input)
    }
}

impl From<(Vec<Statement>, Option<LastStatement>)> for Block {
    fn from((statements, last_statement): (Vec<Statement>, Option<LastStatement>)) -> Self {
        Self::new(statements, last_statement)
    }
}

impl builders::LastStatement<Expression> for LastStatement {
    fn break_statement() -> Self {
        Self::Break
    }

    fn return_statement(expressions: Vec<Expression>) -> Self {
        Self::Return(expressions)
    }
}

impl From<(Vec<Variable>, Vec<Expression>)> for AssignStatement {
    fn from((variables, values): (Vec<Variable>, Vec<Expression>)) -> Self {
        Self::new(variables, values)
    }
}

impl builders::Variable<Prefix> for Variable {
    fn try_from_prefix(prefix: Prefix) -> Result<Self, Prefix> {
        match prefix {
            Prefix::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            Prefix::Field(field) => Ok(Self::Field(field)),
            Prefix::Index(index) => Ok(Self::Index(index)),
            _ => Err(prefix)
        }
    }
}

impl From<Block> for DoStatement {
    fn from(block: Block) -> Self {
        DoStatement::new(block)
    }
}

impl From<(String, Vec<String>, Option<String>, Block, Vec<String>, bool)> for FunctionStatement {
    fn from((name, field_names, method, block, parameters, is_variadic): (String, Vec<String>, Option<String>, Block, Vec<String>, bool)) -> Self {
        let name = FunctionName::new(name, field_names, method);

        Self::new(name, block, parameters, is_variadic)
    }
}

impl From<(Vec<String>, Vec<Expression>, Block)> for GenericForStatement {
    fn from((identifiers, expressions, block): (Vec<String>, Vec<Expression>, Block)) -> Self {
        Self::new(identifiers, expressions, block)
    }
}

impl From<(Expression, Block)> for IfBranch {
    fn from((condition, block): (Expression, Block)) -> Self {
        Self::new(condition, block)
    }
}

impl From<(Vec<(Expression, Block)>, Option<Block>)> for IfStatement {
    fn from((branches, else_block): (Vec<(Expression, Block)>, Option<Block>)) -> Self {
        Self::new(
            branches.into_iter().map(IfBranch::from).collect(),
            else_block,
        )
    }
}

impl From<(Vec<String>, Vec<Expression>)> for LocalAssignStatement {
    fn from((variables, values): (Vec<String>, Vec<Expression>)) -> Self {
        Self::new(variables, values)
    }
}

impl From<(String, Vec<String>, bool, Block)> for LocalFunctionStatement {
    fn from((identifier, parameters, is_variadic, block): (String, Vec<String>, bool, Block)) -> Self {
        Self::new(identifier, block, parameters, is_variadic)
    }
}

impl From<(String, Expression, Expression, Option<Expression>, Block)> for NumericForStatement {
    fn from((identifier, start, end, step, block): (String, Expression, Expression, Option<Expression>, Block)) -> Self {
        Self::new(identifier, start, end, step, block)
    }
}

impl From<(Expression, Block)> for RepeatStatement {
    fn from((condition, block): (Expression, Block)) -> Self {
        Self::new(block, condition)
    }
}

impl From<(Expression, Block)> for WhileStatement {
    fn from((condition, block): (Expression, Block)) -> Self {
        Self::new(block, condition)
    }
}

impl builders::Expression for Expression {
    fn false_expression() -> Self { Self::False }
    fn true_expression() -> Self { Self::True }
    fn nil_expression() -> Self { Self::Nil }
    fn variable_arguments() -> Self { Self::VariableArguments }
    fn parenthese(expression: Self) -> Self { Self::Parenthese(Box::new(expression)) }
}

impl From<(Expression, BinaryOperator, Expression)> for BinaryExpression {
    fn from((left, operator, right): (Expression, BinaryOperator, Expression)) -> Self {
        Self::new(operator, left, right)
    }
}

impl builders::BinaryOperator for BinaryOperator {
    fn and() -> Self { Self::And }
    fn or() -> Self { Self::Or }
    fn equal() -> Self { Self::Equal }
    fn not_equal() -> Self { Self::NotEqual }
    fn lower_than() -> Self { Self::LowerThan }
    fn lower_or_equal_than() -> Self { Self::LowerOrEqualThan }
    fn greather_than() -> Self { Self::GreaterThan }
    fn greather_or_equal_than() -> Self { Self::GreaterOrEqualThan }
    fn plus() -> Self { Self::Plus }
    fn minus() -> Self { Self::Minus }
    fn asterisk() -> Self { Self::Asterisk }
    fn slash() -> Self { Self::Slash }
    fn percent() -> Self { Self::Percent }
    fn caret() -> Self { Self::Caret }
    fn concat() -> Self { Self::Concat }
}

impl From<(Prefix, String)> for FieldExpression {
    fn from((prefix, field): (Prefix, String)) -> Self {
        Self::new(prefix, field)
    }
}

impl builders::Arguments<Expression, TableExpression> for Arguments {
    fn from_string(string: String) -> Self {
        Self::String(StringExpression::from(string))
    }
    fn from_table(table: TableExpression) -> Self { Self::Table(table) }

    fn from_expressions(expressions: Vec<Expression>) -> Self {
        Self::Tuple(expressions)
    }
}

impl From<(Prefix, Arguments, Option<String>)> for FunctionCall {
    fn from((prefix, arguments, method): (Prefix, Arguments, Option<String>)) -> Self {
        Self::new(prefix, arguments, method)
    }
}

impl From<(Vec<String>, bool, Block)> for FunctionExpression {
    fn from((parameters, is_variadic, block): (Vec<String>, bool, Block)) -> Self {
        Self::new(block, parameters, is_variadic)
    }
}

impl From<(Prefix, Expression)> for IndexExpression {
    fn from((prefix, index): (Prefix, Expression)) -> Self {
        Self::new(prefix, index)
    }
}

impl From<&str> for NumberExpression {
    fn from(value: &str) -> Self {
        match value.parse() {
            Ok(value) => value,
            Err(error) => panic!("{}", error),
        }
    }
}

impl From<String> for NumberExpression {
    fn from(value: String) -> Self {
        NumberExpression::from(value.as_str())
    }
}

impl builders::Prefix<Expression, FunctionCall, FieldExpression, IndexExpression> for Prefix {
    fn from_name(name: String) -> Self { Self::Identifier(name) }
    fn from_parenthese(expression: Expression) -> Self { Self::Parenthese(expression) }
    fn from_call(call: FunctionCall) -> Self { Self::Call(call) }
    fn from_field(field: FieldExpression) -> Self { Self::Field(Box::new(field)) }
    fn from_index(index: IndexExpression) -> Self { Self::Index(Box::new(index)) }
}

impl TryInto<FunctionCall> for Prefix {
    type Error = ();

    fn try_into(self) -> Result<FunctionCall, ()> {
        match self {
            Self::Call(call) => Ok(call),
            _ => Err(()),
        }
    }
}

impl From<String> for StringExpression {
    fn from(string: String) -> Self {
        Self::new(string).expect("invalid parsed string")
    }
}

impl From<(String, Expression)> for TableEntry {
    fn from((field, value): (String, Expression)) -> Self {
        Self::Field(field, value)
    }
}

impl builders::TableEntry<Expression> for TableEntry {
    fn from_value(value: Expression) -> Self { Self::Value(value) }
    fn from_field(field: String, value: Expression) -> Self { Self::Field(field, value) }
    fn from_index(key: Expression, value: Expression) -> Self { Self::Index(key, value) }
}

impl From<Vec<TableEntry>> for TableExpression {
    fn from(entries: Vec<TableEntry>) -> Self {
        Self::new(entries)
    }
}

impl From<(UnaryOperator, Expression)> for UnaryExpression {
    fn from((operator, expression): (UnaryOperator, Expression)) -> Self {
        Self::new(operator, expression)
    }
}

impl builders::UnaryOperator for UnaryOperator {
    fn minus() -> Self { Self::Minus }
    fn length() -> Self { Self::Length }
    fn not() -> Self { Self::Not }
}
