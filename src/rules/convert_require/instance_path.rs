use crate::nodes::{FieldExpression, Identifier, Prefix};

use super::RobloxIndexStyle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstancePath {
    root: InstancePathRoot,
    components: Vec<InstancePathComponent>,
}

impl InstancePath {
    pub(crate) fn from_root() -> Self {
        Self {
            root: InstancePathRoot::Root,
            components: Vec::new(),
        }
    }

    pub(crate) fn from_script() -> Self {
        Self {
            root: InstancePathRoot::Script,
            components: Vec::new(),
        }
    }

    pub(crate) fn parent(&mut self) {
        self.components.push(InstancePathComponent::Parent);
    }

    pub(crate) fn child(&mut self, child_name: impl Into<String>) {
        self.components
            .push(InstancePathComponent::Child(child_name.into()));
    }

    pub(crate) fn convert(&self, index_style: &RobloxIndexStyle) -> Prefix {
        let mut prefix = match &self.root {
            InstancePathRoot::Root => datamodel_identifier().into(),
            InstancePathRoot::Script => script_identifier().into(),
        };

        for component in self.components.iter() {
            match component {
                InstancePathComponent::Parent => {
                    prefix = get_parent_instance(prefix);
                }
                InstancePathComponent::Child(child_name) => {
                    prefix = index_style.index(prefix, child_name);
                }
            }
        }

        prefix
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InstancePathRoot {
    Root,
    Script,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InstancePathComponent {
    Parent,
    Child(String),
}

pub(crate) fn script_identifier() -> Identifier {
    Identifier::new("script")
}

pub(crate) fn datamodel_identifier() -> Identifier {
    Identifier::new("game")
}

pub(crate) fn get_parent_instance(instance: impl Into<Prefix>) -> Prefix {
    FieldExpression::new(instance.into(), "Parent").into()
}
