use std::ops;

use crate::{
    nodes::Identifier,
    process::{IdentifierTracker, NodeProcessor},
};

/// A processor to find usage of a given variable.
pub(crate) struct FindUsage<'a> {
    variable: &'a str,
    usage_found: bool,
    identifier_tracker: IdentifierTracker,
}

impl<'a> ops::Deref for FindUsage<'a> {
    type Target = IdentifierTracker;

    fn deref(&self) -> &Self::Target {
        &self.identifier_tracker
    }
}

impl<'a> ops::DerefMut for FindUsage<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identifier_tracker
    }
}

impl<'a> FindUsage<'a> {
    pub fn new(variable: &'a str) -> Self {
        Self {
            variable,
            usage_found: false,
            identifier_tracker: Default::default(),
        }
    }

    #[inline]
    pub fn has_found_usage(&self) -> bool {
        self.usage_found
    }
}

impl<'a> NodeProcessor for FindUsage<'a> {
    fn process_variable_expression(&mut self, variable: &mut Identifier) {
        if !self.usage_found && variable.get_name() == self.variable {
            self.usage_found = !self.is_identifier_used(self.variable);
        }
    }
}
