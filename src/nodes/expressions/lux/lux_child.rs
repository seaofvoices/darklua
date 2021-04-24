use crate::nodes::{
    Expression,
    LUXElement,
    LUXFragment,
};
use luaparser::builders;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXChild {
    LUXElement(LUXElement),
    LUXFragment(LUXFragment),
    Expression(Option<Expression>),
    ExpandedExpression(Expression),
}

impl builders::LUXChild<Expression, LUXElement, LUXFragment> for LUXChild {
    fn from_element(element: LUXElement) -> Self {
        Self::LUXElement(element)
    }

    fn from_fragment(fragment: LUXFragment) -> Self {
        Self::LUXFragment(fragment)
    }

    fn from_expression(expression: Expression) -> Self {
        Self::Expression(Some(expression))
    }

    fn from_expanded_expression(expression: Expression) -> Self {
        Self::ExpandedExpression(expression)
    }

    fn empty() -> Self {
        Self::Expression(None)
    }
}
