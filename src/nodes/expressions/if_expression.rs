use std::cmp::Ordering;

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

    pub fn get_expression(&self, index: usize) -> Option<&Expression> {
        let result = match index {
            0 => self.get_condition(),
            1 => self.get_result(),
            _ => {
                let branch_index = (index / 2).saturating_sub(1);
                match branch_index.cmp(&self.branches.len()) {
                    Ordering::Less => {
                        let branch = self.branches.get(branch_index)?;
                        if index % 2 == 0 {
                            branch.get_condition()
                        } else {
                            branch.get_result()
                        }
                    }
                    Ordering::Equal => self.get_else_result(),
                    Ordering::Greater => return None,
                }
            }
        };
        Some(result)
    }

    #[inline]
    pub fn iter_mut_branches(&mut self) -> impl Iterator<Item = &mut ElseIfExpressionBranch> {
        self.branches.iter_mut()
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
        self.branches
            .iter_mut()
            .for_each(ElseIfExpressionBranch::clear_comments);
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
        self.branches
            .iter_mut()
            .for_each(ElseIfExpressionBranch::clear_whitespaces);
    }
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

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfExpressionTokens {
    pub r#if: Token,
    pub then: Token,
    pub r#else: Token,
}

impl IfExpressionTokens {
    pub fn clear_comments(&mut self) {
        self.r#if.clear_comments();
        self.then.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#if.clear_whitespaces();
        self.then.clear_whitespaces();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElseIfExpressionBranchTokens {
    pub elseif: Token,
    pub then: Token,
}

impl ElseIfExpressionBranchTokens {
    pub fn clear_comments(&mut self) {
        self.elseif.clear_comments();
        self.then.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.elseif.clear_whitespaces();
        self.then.clear_whitespaces();
    }
}
