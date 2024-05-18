use crate::nodes::Token;

use super::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfExpression {
    condition: Expression,
    result: Expression,
    else_result: Expression,
    branches: Vec<ElseIfExpressionBranch>,
    tokens: Option<IfExpressionTokens>,
}

impl IfExpression {
    pub fn new<E: Into<Expression>, E2: Into<Expression>, E3: Into<Expression>>(
        condition: E,
        result: E2,
        else_result: E3,
    ) -> Self {
        Self {
            condition: condition.into(),
            result: result.into(),
            else_result: else_result.into(),
            branches: Vec::new(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: IfExpressionTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn with_branch<E: Into<Expression>, E2: Into<Expression>>(
        mut self,
        condition: E,
        result: E2,
    ) -> Self {
        self.branches
            .push(ElseIfExpressionBranch::new(condition, result));
        self
    }

    #[inline]
    pub fn push_branch(&mut self, branch: ElseIfExpressionBranch) {
        self.branches.push(branch);
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: IfExpressionTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&IfExpressionTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_condition(&self) -> &Expression {
        &self.condition
    }

    #[inline]
    pub fn mutate_condition(&mut self) -> &mut Expression {
        &mut self.condition
    }

    #[inline]
    pub fn get_result(&self) -> &Expression {
        &self.result
    }

    #[inline]
    pub fn mutate_result(&mut self) -> &mut Expression {
        &mut self.result
    }

    #[inline]
    pub fn get_else_result(&self) -> &Expression {
        &self.else_result
    }

    #[inline]
    pub fn mutate_else_result(&mut self) -> &mut Expression {
        &mut self.else_result
    }

    #[inline]
    pub fn has_elseif_branch(&self) -> bool {
        !self.branches.is_empty()
    }

    #[inline]
    pub fn iter_branches(&self) -> impl Iterator<Item = &ElseIfExpressionBranch> {
        self.branches.iter()
    }

    #[inline]
    pub fn clear_elseif_branches(&mut self) {
        self.branches.clear();
    }

    #[inline]
    pub fn retain_elseif_branches_mut(
        &mut self,
        filter: impl FnMut(&mut ElseIfExpressionBranch) -> bool,
    ) {
        self.branches.retain_mut(filter);
    }

    pub fn remove_branch(&mut self, index: usize) -> Option<ElseIfExpressionBranch> {
        if index < self.branches.len() {
            Some(self.branches.remove(index))
        } else {
            None
        }
    }

    #[inline]
    pub fn iter_mut_branches(&mut self) -> impl Iterator<Item = &mut ElseIfExpressionBranch> {
        self.branches.iter_mut()
    }

    super::impl_token_fns!(iter = [tokens, branches]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElseIfExpressionBranch {
    condition: Expression,
    result: Expression,
    tokens: Option<ElseIfExpressionBranchTokens>,
}

impl ElseIfExpressionBranch {
    pub fn new<E: Into<Expression>, E2: Into<Expression>>(condition: E, result: E2) -> Self {
        Self {
            condition: condition.into(),
            result: result.into(),
            tokens: None,
        }
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: ElseIfExpressionBranchTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&ElseIfExpressionBranchTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_condition(&self) -> &Expression {
        &self.condition
    }

    #[inline]
    pub fn mutate_condition(&mut self) -> &mut Expression {
        &mut self.condition
    }

    #[inline]
    pub fn get_result(&self) -> &Expression {
        &self.result
    }

    #[inline]
    pub fn mutate_result(&mut self) -> &mut Expression {
        &mut self.result
    }

    pub fn into_expressions(self) -> (Expression, Expression) {
        (self.condition, self.result)
    }

    super::impl_token_fns!(iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfExpressionTokens {
    pub r#if: Token,
    pub then: Token,
    pub r#else: Token,
}

impl IfExpressionTokens {
    super::impl_token_fns!(target = [r#if, then, r#else]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElseIfExpressionBranchTokens {
    pub elseif: Token,
    pub then: Token,
}

impl ElseIfExpressionBranchTokens {
    super::impl_token_fns!(target = [elseif, then]);
}
