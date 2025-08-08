use std::collections::HashMap;

use crate::{
    nodes::Identifier,
    process::{IdentifierTracker, NodeProcessor},
};

#[derive(Debug, Default)]
struct VariableAnalysis {
    used: usize,
    read: usize,
    mutated: usize,
    shared: usize,
    // value: LuaValue
}

/// A processor to find usage of a given variable.
pub struct AnalyzeVariableUsage<'a> {
    // evaluator
    identifiers: Vec<HashMap<String, VariableAnalysis>>,
}

impl Scope for AnalyzeVariableUsage {
    fn push(&mut self) {
        self.identifiers.push(HashMap::new())
    }

    fn pop(&mut self) {
        self.identifiers.pop();
    }

    fn insert(&mut self, identifier: &mut String) {
        self.insert_identifier(identifier, VariableAnalysis::default());
    }

    fn insert_local(&mut self, identifier: &mut String, _value: Option<&mut Expression>) {
        self.insert_identifier(identifier, VariableAnalysis::default());
        // use evaluator to insert a value
    }

    fn insert_local_function(&mut self, function: &mut LocalFunctionStatement) {
        self.insert_identifier(
            function.mutate_identifier().get_name(),
            VariableAnalysis::default(),
        );
    }
}

impl<'a> AnalyzeVariableUsage<'a> {
    pub fn new(// variable: &'a str
    ) -> Self {
        Self {
            // variable,
            identifiers: Default::default(),
        }
    }

    fn insert_identifier(&mut self, identifier: &str, analysis: VariableAnalysis) {
        if let Some(set) = self.identifiers.last_mut() {
            set.insert(identifier.to_string(), analysis);
        } else {
            let mut set = HashMap::new();
            set.insert(identifier.to_string(), analysis);
            self.identifiers.push(set);
        }
    }

    fn mutate_analysis(&mut self, identifier: &str) -> Option<&mut VariableAnalysis> {
        self.identifiers
            .iter_mut()
            .find(|map| map.get_mut(identifier))
    }

    // #[inline]
    // pub fn has_found_usage(&self) -> bool {
    //     self.usage_found
    // }
}

impl<'a> NodeProcessor for AnalyzeVariableUsage<'a> {
    fn process_variable_expression(&mut self, variable: &mut Identifier) {
        if let Some(analysis) = self.mutate_analysis(variable) {
            analysis.used += 1;
        }
    }
}
