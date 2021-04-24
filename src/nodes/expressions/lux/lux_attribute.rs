use crate::nodes::{
    Expression,
    LUXNamespacedName,
    LUXElement,
    LUXFragment,
};
use luaparser::builders;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXAttributeName {
    Identifier(String),
    NamespacedName(LUXNamespacedName),
}

impl From<String> for LUXAttributeName {
    fn from(identifier: String) -> Self {
        Self::Identifier(identifier)
    }
}

impl From<LUXNamespacedName> for LUXAttributeName {
    fn from(name: LUXNamespacedName) -> Self {
        Self::NamespacedName(name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXAttributeValue {
    DoubleQuoteString(String),
    SingleQuoteString(String),
    LuaExpression(Expression),
    LUXElement(LUXElement),
    LUXFragment(LUXFragment),
}

impl builders::LUXAttributeValue<Expression, LUXElement, LUXFragment> for LUXAttributeValue {
    fn from_double_quote_string(string: String) -> Self {
        Self::DoubleQuoteString(string)
    }

    fn from_single_quote_string(string: String) -> Self {
        Self::SingleQuoteString(string)
    }

    fn from_expression(expression: Expression) -> Self {
        Self::LuaExpression(expression)
    }

    fn from_element(element: LUXElement) -> Self {
        Self::LUXElement(element)
    }

    fn from_fragment(fragment: LUXFragment) -> Self {
        Self::LUXFragment(fragment)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedAttribute {
    name: LUXAttributeName,
    value: Option<LUXAttributeValue>,
}

impl NamedAttribute {
    #[inline]
    pub fn get_name(&self) -> &LUXAttributeName {
        &self.name
    }

    #[inline]
    pub fn get_value(&self) -> &Option<LUXAttributeValue> {
        &self.value
    }

    #[inline]
    pub fn mutate_value(&mut self) -> &mut Option<LUXAttributeValue> {
        &mut self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXAttribute {
    Named(NamedAttribute),
    Spread(Expression),
}

impl From<(LUXAttributeName, Option<LUXAttributeValue>)> for LUXAttribute {
    fn from((name, value): (LUXAttributeName, Option<LUXAttributeValue>)) -> Self {
        Self::Named(NamedAttribute {
            name,
            value,
        })
    }
}

impl From<Expression> for LUXAttribute {
    fn from(expression: Expression) -> Self {
        Self::Spread(expression)
    }
}
