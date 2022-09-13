use crate::{
    nodes::{AnyNodeRef, Expression},
    process::path::{NodePath, NodePathBuf},
};

pub struct NodeStack<'a> {
    stack: Vec<(AnyNodeRef<'a>, NodePathBuf)>,
}

impl<'a> NodeStack<'a> {
    pub fn new(node: impl Into<AnyNodeRef<'a>>) -> Self {
        Self {
            stack: vec![(node.into(), NodePathBuf::default())],
        }
    }

    pub fn push(&mut self, node: impl Into<AnyNodeRef<'a>>, path: NodePathBuf) {
        self.stack.push((node.into(), path));
    }

    pub fn push_expressions(
        &mut self,
        iterator: impl Iterator<Item = &'a Expression>,
        path: &NodePathBuf,
    ) {
        for (index, item) in iterator.enumerate() {
            self.stack.push((item.into(), path.join_expression(index)));
        }
    }

    pub fn pop(&mut self) -> Option<(AnyNodeRef<'a>, NodePathBuf)> {
        self.stack.pop()
    }
}
