use crate::{nodes::Identifier, process::NodeProcessor};

/// A processor to find usage of a given set of identifiers.
///
/// # Example
/// The following example illustrate how you can use this processor to find usage of the
/// `foo` variable.
/// ```
/// # use darklua_core::nodes::Expression;
/// # use darklua_core::process::processors::FindVariables;
/// # use darklua_core::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
/// let variables = vec!["foo".to_owned()];
/// let mut find_foo = FindVariables::from(&variables);
///
/// let mut foo_expression = Expression::identifier("foo");
/// DefaultVisitor::visit_expression(&mut foo_expression, &mut find_foo);
///
/// assert!(find_foo.has_found_usage());
/// ```
/// If you pass a node that does not contain the given variable, the processor will return
/// false when calling the `has_found_usage()` method.
/// ```
/// # use darklua_core::nodes::Expression;
/// # use darklua_core::process::processors::FindVariables;
/// # use darklua_core::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
/// # let variables = vec!["foo".to_owned()];
/// # let mut find_foo = FindVariables::from(&variables);
/// let mut bar_expression = Expression::identifier("bar");
/// DefaultVisitor::visit_expression(&mut bar_expression, &mut find_foo);
///
/// assert!(!find_foo.has_found_usage());
/// ```
pub struct FindVariables<'a> {
    variables: Vec<&'a String>,
    usage_found: bool,
}

impl<'a> FindVariables<'a> {
    #[inline]
    pub fn has_found_usage(&self) -> bool {
        self.usage_found
    }
}

impl<'a> From<&'a Vec<String>> for FindVariables<'a> {
    fn from(variables: &'a Vec<String>) -> Self {
        Self {
            variables: variables.iter().collect(),
            usage_found: false,
        }
    }
}

impl<'a> From<&'a Vec<Identifier>> for FindVariables<'a> {
    fn from(variables: &'a Vec<Identifier>) -> Self {
        Self {
            variables: variables
                .iter()
                .map(|variable| variable.get_name())
                .collect(),
            usage_found: false,
        }
    }
}

impl<'a> NodeProcessor for FindVariables<'a> {
    fn process_variable_expression(&mut self, variable: &mut Identifier) {
        if !self.usage_found {
            let name = variable.get_name();
            self.usage_found = self.variables.iter().any(|v| *v == name)
        }
    }
}
