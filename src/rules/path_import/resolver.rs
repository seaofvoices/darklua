use std::{fmt::Debug, path::Path};

use crate::nodes::Expression;

pub trait Resolver: Debug {
    fn resolve(&self, path: &Path) -> Option<Expression>;
}
