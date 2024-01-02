use crate::nodes::{
    BinaryExpression, BinaryOperator, Block, DoStatement, Expression, LocalAssignStatement,
    Statement,
};

pub(crate) fn expressions_as_statement(expressions: Vec<Expression>) -> Statement {
    let mut statements: Vec<Statement> = Vec::new();

    for value in expressions {
        match value {
            Expression::Call(call) => {
                statements.push((*call).into());
            }
            value => {
                if let Some(assign) = statements.last_mut().and_then(|statement| match statement {
                    Statement::LocalAssign(assign) => Some(assign),
                    _ => None,
                }) {
                    assign.push_value(value);
                } else {
                    statements.push(
                        LocalAssignStatement::from_variable("_")
                            .with_value(value)
                            .into(),
                    );
                }
            }
        }
    }

    if statements.len() == 1 {
        if let Statement::Call(call) = statements.pop().unwrap() {
            return call.into();
        }
    }

    DoStatement::new(Block::new(statements, None)).into()
}

pub(crate) fn expressions_as_expression(expressions: Vec<Expression>) -> Expression {
    if expressions.is_empty() {
        return Expression::nil();
    }

    if expressions.len() == 1 {
        BinaryExpression::new(
            BinaryOperator::And,
            expressions.into_iter().next().unwrap(),
            Expression::nil(),
        )
        .into()
    } else {
        expressions
            .into_iter()
            .rfold(Expression::nil(), |current, value| {
                BinaryExpression::new(
                    BinaryOperator::And,
                    BinaryExpression::new(BinaryOperator::Or, value, true),
                    current,
                )
                .into()
            })
    }
}
