use crate::process::path::NodePath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatementSpan {
    path: NodePath,
    length: usize,
}

impl StatementSpan {
    pub fn path(&self) -> &NodePath {
        &self.path
    }

    pub fn len(&self) -> usize {
        self.length + 1
    }
}

impl NodePath {
    pub fn span(self, length: usize) -> StatementSpan {
        debug_assert!(length > 0, "StatementSpan length must always be above zero");
        StatementSpan { path: self, length }
    }
}

impl From<NodePath> for StatementSpan {
    fn from(path: NodePath) -> Self {
        Self { path, length: 1 }
    }
}
