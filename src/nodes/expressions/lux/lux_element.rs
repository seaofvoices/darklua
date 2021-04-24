use crate::nodes::{
    LUXAttribute,
    LUXChild,
    LUXElementName,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LUXSelfClosingElement {
    name: LUXElementName,
    attributes: Vec<LUXAttribute>,
}

impl LUXSelfClosingElement {
    #[inline]
    pub fn get_name(&self) -> &LUXElementName {
        &self.name
    }

    #[inline]
    pub fn get_attributes(&self) -> &Vec<LUXAttribute> {
        &self.attributes
    }

    #[inline]
    pub fn mutate_attributes(&mut self) -> &mut Vec<LUXAttribute> {
        &mut self.attributes
    }
}

impl From<(LUXElementName, Vec<LUXAttribute>)> for LUXSelfClosingElement {
    fn from((name, attributes): (LUXElementName, Vec<LUXAttribute>)) -> Self {
        Self {
            name,
            attributes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LUXOpeningElement {
    name: LUXElementName,
    attributes: Vec<LUXAttribute>,
}

impl LUXOpeningElement {
    #[inline]
    pub fn get_name(&self) -> &LUXElementName {
        &self.name
    }

    #[inline]
    pub fn get_attributes(&self) -> &Vec<LUXAttribute> {
        &self.attributes
    }

    #[inline]
    pub fn mutate_attributes(&mut self) -> &mut Vec<LUXAttribute> {
        &mut self.attributes
    }
}

impl From<(LUXElementName, Vec<LUXAttribute>)> for LUXOpeningElement {
    fn from((name, attributes): (LUXElementName, Vec<LUXAttribute>)) -> Self {
        Self {
            name,
            attributes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LUXClosingElement {
    name: LUXElementName,
}

impl From<LUXElementName> for LUXClosingElement {
    fn from(name: LUXElementName) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LUXOpenCloseElement {
    opening: LUXOpeningElement,
    closing: LUXClosingElement,
    children: Vec<LUXChild>,
}

impl LUXOpenCloseElement {
    #[inline]
    pub fn get_name(&self) -> &LUXElementName {
        &self.opening.get_name()
    }

    #[inline]
    pub fn get_attributes(&self) -> &Vec<LUXAttribute> {
        &self.opening.get_attributes()
    }

    #[inline]
    pub fn mutate_attributes(&mut self) -> &mut Vec<LUXAttribute> {
        self.opening.mutate_attributes()
    }

    #[inline]
    pub fn mutate_children(&mut self) -> &mut Vec<LUXChild> {
        &mut self.children
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXElement {
    SelfClosingElement(LUXSelfClosingElement),
    Element(LUXOpenCloseElement),
}

impl LUXElement {
    pub fn get_name(&self) -> &LUXElementName {
        match self {
            Self::SelfClosingElement(element) => element.get_name(),
            Self::Element(element) => element.get_name(),
        }
    }

    pub fn get_attributes(&self) -> &Vec<LUXAttribute> {
        match self {
            Self::SelfClosingElement(element) => element.get_attributes(),
            Self::Element(element) => element.get_attributes(),
        }
    }
}

impl From<LUXSelfClosingElement> for LUXElement {
    fn from(element: LUXSelfClosingElement) -> Self {
        Self::SelfClosingElement(element)
    }
}

impl From<(LUXOpeningElement, LUXClosingElement, Vec<LUXChild>)> for LUXElement {
    fn from((opening, closing, children): (LUXOpeningElement, LUXClosingElement, Vec<LUXChild>)) -> Self {
        Self::Element(LUXOpenCloseElement {
            opening,
            closing,
            children,
        })
    }
}
