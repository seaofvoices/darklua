mod lux_attribute;
mod lux_child;
mod lux_element_name;
mod lux_element;
mod lux_fragment;

pub use lux_attribute::*;
pub use lux_child::*;
pub use lux_element_name::*;
pub use lux_element::*;
pub use lux_fragment::*;

use crate::nodes::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXExpression {
    LUXElement(LUXElement),
    LUXFragment(LUXFragment),
}

impl From<LUXElement> for LUXExpression {
    fn from(element: LUXElement) -> Self {
        Self::LUXElement(element)
    }
}

impl From<LUXFragment> for LUXExpression {
    fn from(fragment: LUXFragment) -> Self {
        Self::LUXFragment(fragment)
    }
}

impl Into<Expression> for LUXExpression {
    fn into(self) -> Expression {
        Expression::LUX(self)
    }
}
