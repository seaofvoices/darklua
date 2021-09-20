use darklua_core::nodes::*;
use rand::{thread_rng, Rng};
use rand_distr::{Alphanumeric, Normal, Poisson};
use std::iter;

fn generated_identifier() -> String {
    let poisson = Poisson::new(3.0).unwrap();

    let mut rng = thread_rng();
    let length: u64 = rng.sample(poisson);

    let identifier: String = (0..1 + length)
        .map(|i| loop {
            let character = rng.sample(Alphanumeric);

            if i != 0 || !character.is_digit(10) {
                return character;
            }
        })
        .collect();

    match identifier.as_ref() {
        "and" | "break" | "do" | "else" | "elseif" | "end" | "false" | "for" | "function"
        | "if" | "in" | "local" | "nil" | "not" | "or" | "repeat" | "return" | "then" | "true"
        | "until" | "while" => generated_identifier(),
        _ => identifier,
    }
}

#[inline]
fn generated_identifiers(length: usize) -> Vec<String> {
    iter::repeat(())
        .take(length)
        .map(|()| generated_identifier())
        .collect()
}

#[inline]
fn function_param_length() -> usize {
    let normal = Normal::new(0.0, 2.5).unwrap();
    (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn function_name_field_length() -> usize {
    let normal = Normal::new(0.0, 1.0).unwrap();
    (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn assign_variables_length() -> usize {
    let normal = Normal::new(0.0, 1.0).unwrap();
    1 + (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn local_assign_values_length() -> usize {
    let normal = Normal::new(1.0, 1.0).unwrap();
    (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn assign_values_length() -> usize {
    let normal = Normal::new(0.0, 1.0).unwrap();
    1 + (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn generic_for_expression_length() -> usize {
    let normal = Normal::new(0.0, 1.0).unwrap();
    1 + (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn if_branch_count() -> usize {
    let normal = Normal::new(0.0, 1.5).unwrap();
    1 + (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn table_length() -> usize {
    let normal = Normal::new(1.0, 1.5).unwrap();
    (thread_rng().sample(normal) as f64).abs().floor() as usize
}

#[inline]
fn generate_expressions(length: usize, context: &mut FuzzContext) -> Vec<Expression> {
    iter::repeat(())
        .take(length)
        .filter_map(|()| {
            if context.take_expression() {
                Some(Expression::fuzz(context))
            } else {
                None
            }
        })
        .collect()
}

#[inline]
fn generate_at_least_one_expression(length: usize, context: &mut FuzzContext) -> Vec<Expression> {
    (0..length)
        .filter_map(|i| {
            if context.take_expression() || i == 0 {
                Some(Expression::fuzz(context))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct FuzzContext {
    statements_budget: u32,
    expressions_budget: u32,
}

impl FuzzContext {
    pub fn new(statements_budget: u32, expressions_budget: u32) -> Self {
        Self {
            statements_budget,
            expressions_budget,
        }
    }

    pub fn take_statement(&mut self) -> bool {
        if self.statements_budget == 0 {
            false
        } else {
            self.statements_budget -= 1;
            true
        }
    }

    pub fn take_expression(&mut self) -> bool {
        if self.expressions_budget == 0 {
            false
        } else {
            self.expressions_budget -= 1;
            true
        }
    }

    #[inline]
    pub fn can_have_expression(&self, amount: u32) -> bool {
        self.expressions_budget >= amount
    }

    pub fn share_budget(&mut self) -> Self {
        let statement_amount = if self.statements_budget == 0 {
            0
        } else {
            let statement_amount = thread_rng().gen_range(0, self.statements_budget);
            self.statements_budget -= statement_amount;
            statement_amount
        };
        let expression_amount = if self.expressions_budget == 0 {
            0
        } else {
            let expression_amount = thread_rng().gen_range(0, self.expressions_budget);
            self.expressions_budget -= expression_amount;
            expression_amount
        };

        Self::new(statement_amount, expression_amount)
    }
}

pub trait Fuzz<T> {
    fn fuzz(context: &mut FuzzContext) -> T;
}

impl Fuzz<Block> for Block {
    fn fuzz(context: &mut FuzzContext) -> Self {
        let mut statements = Vec::new();
        let mut last_statement = None;

        if rand::random() {
            if context.take_statement() {
                last_statement.replace(LastStatement::fuzz(context));
            }
        }

        while context.take_statement() {
            statements.push(Statement::fuzz(context));
        }

        Block::new(statements, last_statement)
    }
}

impl Fuzz<Statement> for Statement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        match thread_rng().gen_range(0, 12) {
            0 => AssignStatement::fuzz(context).into(),
            1 => DoStatement::fuzz(&mut context.share_budget()).into(),
            2 => FunctionCall::fuzz(context).into(),
            3 => FunctionStatement::fuzz(&mut context.share_budget()).into(),
            4 => GenericForStatement::fuzz(&mut context.share_budget()).into(),
            5 => IfStatement::fuzz(&mut context.share_budget()).into(),
            6 => LocalAssignStatement::fuzz(&mut context.share_budget()).into(),
            7 => LocalFunctionStatement::fuzz(&mut context.share_budget()).into(),
            8 => NumericForStatement::fuzz(&mut context.share_budget()).into(),
            9 => RepeatStatement::fuzz(&mut context.share_budget()).into(),
            10 => WhileStatement::fuzz(&mut context.share_budget()).into(),
            _ => CompoundAssignStatement::fuzz(&mut context.share_budget()).into(),
        }
    }
}

impl Fuzz<LastStatement> for LastStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        match thread_rng().gen_range(0, 3) {
            0 => Self::Break,
            1 => Self::Continue,
            _ => {
                let normal = Normal::new(0.0, 2.5).unwrap();
                let mut rng = thread_rng();
                let length = (rng.sample(normal) as f64).abs().floor() as usize;

                Self::Return(generate_expressions(length, context))
            }
        }
    }
}

impl Fuzz<AssignStatement> for AssignStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            iter::repeat(())
                .take(assign_variables_length())
                .map(|()| Variable::fuzz(context))
                .collect(),
            generate_at_least_one_expression(assign_values_length(), context),
        )
    }
}

impl Fuzz<CompoundAssignStatement> for CompoundAssignStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            CompoundOperator::fuzz(context),
            Variable::fuzz(context),
            Expression::fuzz(context),
        )
    }
}

impl Fuzz<CompoundOperator> for CompoundOperator {
    fn fuzz(_context: &mut FuzzContext) -> Self {
        use CompoundOperator::*;

        match thread_rng().gen_range(0, 8) {
            1 => Plus,
            2 => Minus,
            3 => Asterisk,
            4 => Slash,
            5 => Percent,
            6 => Caret,
            _ => Concat,
        }
    }
}

impl Fuzz<Variable> for Variable {
    fn fuzz(context: &mut FuzzContext) -> Self {
        use Variable::*;

        if context.can_have_expression(2) {
            match thread_rng().gen_range(0, 3) {
                0 => Identifier(generated_identifier()),
                1 => Field(FieldExpression::fuzz(context).into()),
                _ => Index(IndexExpression::fuzz(context).into()),
            }
        } else {
            Identifier(generated_identifier())
        }
    }
}

impl Fuzz<DoStatement> for DoStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(Block::fuzz(context))
    }
}

impl Fuzz<FunctionStatement> for FunctionStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            FunctionName::fuzz(context),
            Block::fuzz(context),
            generated_identifiers(function_param_length()),
            rand::random(),
        )
    }
}

impl Fuzz<FunctionName> for FunctionName {
    fn fuzz(_context: &mut FuzzContext) -> Self {
        Self::new(
            generated_identifier(),
            generated_identifiers(function_name_field_length()),
            if rand::random() {
                Some(generated_identifier())
            } else {
                None
            },
        )
    }
}

impl Fuzz<GenericForStatement> for GenericForStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            generated_identifiers(1),
            generate_at_least_one_expression(generic_for_expression_length(), context),
            Block::fuzz(context),
        )
    }
}

impl Fuzz<IfStatement> for IfStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            generate_at_least_one_expression(if_branch_count(), context)
                .into_iter()
                .map(|condition| IfBranch::new(condition, Block::fuzz(&mut context.share_budget())))
                .collect(),
            if rand::random() {
                Some(Block::fuzz(&mut context.share_budget()))
            } else {
                None
            },
        )
    }
}

impl Fuzz<LocalAssignStatement> for LocalAssignStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            generated_identifiers(1 + assign_variables_length()),
            generate_expressions(local_assign_values_length(), context),
        )
    }
}

impl Fuzz<LocalFunctionStatement> for LocalFunctionStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            generated_identifier(),
            Block::fuzz(context),
            generated_identifiers(function_param_length()),
            rand::random(),
        )
    }
}

impl Fuzz<NumericForStatement> for NumericForStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            generated_identifier(),
            Expression::fuzz(context),
            Expression::fuzz(context),
            if rand::random() {
                Some(Expression::fuzz(context))
            } else {
                None
            },
            Block::fuzz(context),
        )
    }
}

impl Fuzz<RepeatStatement> for RepeatStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(Block::fuzz(context), Expression::fuzz(context))
    }
}

impl Fuzz<WhileStatement> for WhileStatement {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(Block::fuzz(context), Expression::fuzz(context))
    }
}

impl Fuzz<Expression> for Expression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        use Expression::*;

        context.take_expression();

        if context.can_have_expression(2) {
            match thread_rng().gen_range(0, 15) {
                0 => True,
                1 => False,
                2 => Nil,
                3 => VariableArguments,
                4 => Parenthese(Box::new(Self::fuzz(context))),
                5 => BinaryExpression::fuzz(context).into(),
                6 => FunctionCall::fuzz(context).into(),
                7 => FieldExpression::fuzz(context).into(),
                8 => FunctionExpression::fuzz(context).into(),
                9 => Identifier(generated_identifier()),
                10 => IndexExpression::fuzz(context).into(),
                11 => NumberExpression::fuzz(context).into(),
                12 => StringExpression::fuzz(context).into(),
                13 => TableExpression::fuzz(&mut context.share_budget()).into(),
                _ => UnaryExpression::fuzz(context).into(),
            }
        } else {
            match thread_rng().gen_range(0, 15) {
                0 => True,
                1 => False,
                2 => Nil,
                3 => VariableArguments,
                4 => FunctionCall::fuzz(context).into(),
                5 => FunctionExpression::fuzz(context).into(),
                6 => Identifier(generated_identifier()),
                7 => NumberExpression::fuzz(context).into(),
                8 => StringExpression::fuzz(context).into(),
                _ => TableExpression::fuzz(&mut context.share_budget()).into(),
            }
        }
    }
}

#[inline]
fn get_binary_operator(expression: &Expression) -> Option<BinaryOperator> {
    match expression {
        Expression::Binary(expression) => Some(expression.operator()),
        _ => None,
    }
}

impl Fuzz<BinaryExpression> for BinaryExpression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        let operator = BinaryOperator::fuzz(context);
        let mut left = Expression::fuzz(context);
        let mut right = Expression::fuzz(context);

        if operator.left_needs_parentheses(&left) {
            left = Expression::Parenthese(left.into());
        }

        if operator.right_needs_parentheses(&right) {
            right = Expression::Parenthese(right.into());
        }

        Self::new(operator, left, right)
    }
}

impl Fuzz<BinaryOperator> for BinaryOperator {
    fn fuzz(_context: &mut FuzzContext) -> Self {
        use BinaryOperator::*;

        match thread_rng().gen_range(0, 15) {
            0 => And,
            1 => Or,
            2 => Equal,
            3 => NotEqual,
            4 => LowerThan,
            5 => LowerOrEqualThan,
            6 => GreaterThan,
            7 => GreaterOrEqualThan,
            8 => Plus,
            9 => Minus,
            10 => Asterisk,
            11 => Slash,
            12 => Percent,
            13 => Caret,
            _ => Concat,
        }
    }
}

impl Fuzz<FieldExpression> for FieldExpression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(Prefix::fuzz(context), generated_identifier())
    }
}

impl Fuzz<Arguments> for Arguments {
    fn fuzz(context: &mut FuzzContext) -> Self {
        match thread_rng().gen_range(0, 3) {
            0 => Self::Tuple(generate_expressions(function_param_length(), context)),
            1 => Self::String(StringExpression::fuzz(context)),
            _ => Self::Table(TableExpression::fuzz(context)),
        }
    }
}

impl Fuzz<FunctionCall> for FunctionCall {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            Prefix::fuzz(context),
            Arguments::fuzz(context),
            if rand::random() {
                Some(generated_identifier())
            } else {
                None
            },
        )
    }
}

impl Fuzz<FunctionExpression> for FunctionExpression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            Block::fuzz(&mut context.share_budget()),
            generated_identifiers(function_param_length()),
            rand::random(),
        )
    }
}

impl Fuzz<IndexExpression> for IndexExpression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(Prefix::fuzz(context), Expression::fuzz(context))
    }
}

impl Fuzz<NumberExpression> for NumberExpression {
    fn fuzz(_context: &mut FuzzContext) -> Self {
        match thread_rng().gen_range(0, 4) {
            0 => DecimalNumber::new(thread_rng().gen()).into(),
            1 => HexNumber::new(thread_rng().gen_range(0, 100_000), rand::random()).into(),
            _ => BinaryNumber::new(thread_rng().gen_range(0, 1_000_000), rand::random()).into(),
        }
    }
}

impl Fuzz<Prefix> for Prefix {
    fn fuzz(context: &mut FuzzContext) -> Self {
        use Prefix::*;

        if context.can_have_expression(2) {
            match thread_rng().gen_range(0, 5) {
                0 => Call(FunctionCall::fuzz(context)),
                1 => Field(FieldExpression::fuzz(context).into()),
                2 => Identifier(generated_identifier()),
                3 => Index(IndexExpression::fuzz(context).into()),
                _ => Parenthese(Expression::fuzz(context)),
            }
        } else {
            if rand::random() {
                Call(FunctionCall::fuzz(context))
            } else {
                Identifier(generated_identifier())
            }
        }
    }
}

impl Fuzz<StringExpression> for StringExpression {
    fn fuzz(_context: &mut FuzzContext) -> Self {
        let poisson = Poisson::new(3.0).unwrap();

        let mut rng = thread_rng();
        let length: u64 = rng.sample(poisson);

        const GEN_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                abcdefghijklmnopqrstuvwxyz\
                0123456789\
                ()[]{}=<>.!?,:;+-*/%^|&#";

        Self::from_value::<String>(
            iter::repeat(())
                .take(length as usize)
                .map(|()| GEN_CHARSET[rng.gen_range(0, GEN_CHARSET.len())] as char)
                .collect(),
        )
    }
}

impl Fuzz<TableExpression> for TableExpression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        Self::new(
            iter::repeat(())
                .take(table_length())
                .filter_map(|()| {
                    if context.take_expression() {
                        Some(TableEntry::fuzz(context))
                    } else {
                        None
                    }
                })
                .collect(),
        )
    }
}

impl Fuzz<TableEntry> for TableEntry {
    fn fuzz(context: &mut FuzzContext) -> Self {
        if context.can_have_expression(2) {
            match thread_rng().gen_range(0, 3) {
                0 => Self::Field(generated_identifier(), Expression::fuzz(context)),
                1 => Self::Index(Expression::fuzz(context), Expression::fuzz(context)),
                _ => Self::Value(Expression::fuzz(context)),
            }
        } else {
            if rand::random() {
                Self::Field(generated_identifier(), Expression::fuzz(context))
            } else {
                Self::Value(Expression::fuzz(context))
            }
        }
    }
}

impl Fuzz<UnaryExpression> for UnaryExpression {
    fn fuzz(context: &mut FuzzContext) -> Self {
        let mut expression = Expression::fuzz(context);

        if let Some(inner_operator) = get_binary_operator(&expression) {
            if !inner_operator.precedes_unary_expression() {
                expression = Expression::Parenthese(expression.into());
            }
        }

        Self::new(UnaryOperator::fuzz(context), expression)
    }
}

impl Fuzz<UnaryOperator> for UnaryOperator {
    fn fuzz(_context: &mut FuzzContext) -> Self {
        use UnaryOperator::*;

        match thread_rng().gen_range(0, 3) {
            0 => Length,
            1 => Minus,
            _ => Not,
        }
    }
}
