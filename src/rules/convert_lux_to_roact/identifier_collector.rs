use crate::process::NodeProcessor;

use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct IdentifierCollector {
    identifiers: HashSet<String>,
}

impl IdentifierCollector {
    pub fn try_get_identifier<I: Into<String>>(&self, identifier: I) -> String {
        let mut identifier = identifier.into();
        if self.identifiers.contains(&identifier) {
            identifier.insert(0, '_');
            self.try_get_identifier(identifier)
        } else {
            identifier
        }
    }
}

impl NodeProcessor for IdentifierCollector {
    fn process_variable_expression(&mut self, variable: &mut String) {
        self.identifiers.insert(variable.to_owned());
    }
}
