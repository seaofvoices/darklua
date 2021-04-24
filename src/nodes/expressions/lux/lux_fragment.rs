use crate::nodes::{
    LUXChild,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LUXFragment {
    children: Vec<LUXChild>,
}

impl From<Vec<LUXChild>> for LUXFragment {
    fn from(children: Vec<LUXChild>) -> Self {
        Self { children }
    }
}

impl LUXFragment {
    #[inline]
    pub fn get_children(&self) -> &Vec<LUXChild> {
        &self.children
    }

    #[inline]
    pub fn mutate_children(&mut self) -> &mut Vec<LUXChild> {
        &mut self.children
    }
}
