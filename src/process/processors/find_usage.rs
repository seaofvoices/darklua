use crate::{
    nodes::Identifier,
    process::{IdentifierTracker, NodeProcessor},
};

/// A processor to find usage of a given variable.
pub struct FindVariableUsage<'a> {
    variable: &'a str,
    identifier_tracker: IdentifierTracker,
}

impl<'a> FindVariableUsage<'a> {
    pub fn new(variable: &'a str) -> Self {
        Self {
            variable,
            identifier_tracker: Default::default(),
        }
    }

    // #[inline]
    // pub fn has_found_usage(&self) -> bool {
    //     self.usage_found
    // }
}

impl<'a> NodeProcessor for FindVariableUsage<'a> {
    fn process_variable_expression(&mut self, variable: &mut Identifier) {
        // if !self.usage_found {
        //     let name = variable.get_name();
        //     self.usage_found = self.variables.iter().any(|v| *v == name)
        // }
    }
}
