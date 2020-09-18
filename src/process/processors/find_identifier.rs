use crate::process::NodeProcessorMut;

/// A processor to find usage of a given set of identifiers.
///
/// # Example
/// The following example illustrate how you can use this processor to find usage of the
/// `foo` variable.
/// ```
/// # use darklua_core::nodes::Expression;
/// # use darklua_core::process::processors::FindVariables;
/// # use darklua_core::process::{DefaultVisitorMut, NodeProcessorMut, NodeVisitorMut};
/// let variables = vec!["foo".to_owned()];
/// let mut find_foo = FindVariables::from(&variables);
///
/// let mut foo_expression = Expression::Identifier("foo".to_owned());
/// DefaultVisitorMut::visit_expression(&mut foo_expression, &mut find_foo);
///
/// assert!(find_foo.has_found_usage());
/// ```
/// If you pass a node that does not contain the given variable, the processor will return
/// false when calling the `has_found_usage()` method.
/// ```
/// # use darklua_core::nodes::Expression;
/// # use darklua_core::process::processors::FindVariables;
/// # use darklua_core::process::{DefaultVisitorMut, NodeProcessorMut, NodeVisitorMut};
/// # let variables = vec!["foo".to_owned()];
/// # let mut find_foo = FindVariables::from(&variables);
/// let mut bar_expression = Expression::Identifier("bar".to_owned());
/// DefaultVisitorMut::visit_expression(&mut bar_expression, &mut find_foo);
///
/// assert!(!find_foo.has_found_usage());
/// ```
pub struct FindVariables<'a> {
    variables: &'a Vec<String>,
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
            variables,
            usage_found: false,
        }
    }
}

impl<'a> NodeProcessorMut for FindVariables<'a> {
    fn process_variable_expression(&mut self, variable: &mut String) {
        if !self.usage_found {
            self.usage_found = self.variables.iter().any(|v| v == variable)
        }
    }
}
