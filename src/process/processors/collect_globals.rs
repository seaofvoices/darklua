use std::collections::HashSet;

use crate::{
    nodes::{Expression, FunctionAssignment, Identifier, TypeField},
    process::{NodeProcessor, Scope},
};

#[derive(Debug)]
pub struct CollectGlobalsProcessor {
    scopes: Vec<HashSet<String>>,
    globals: HashSet<String>,
}

impl CollectGlobalsProcessor {
    pub fn new() -> Self {
        Self {
            scopes: Default::default(),
            globals: Default::default(),
        }
    }

    pub(crate) fn add(&mut self, identifier: &str) {
        if self.scopes.is_empty() {
            self.scopes.push(HashSet::new());
        }

        let current = self.scopes.last_mut().unwrap();
        if !current.contains(identifier) {
            current.insert(identifier.to_owned());
        }
    }

    pub fn is_declared(&self, identifier: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(identifier))
    }

    pub fn iter_globals(&self) -> impl Iterator<Item = &str> {
        self.globals.iter().map(String::as_str)
    }

    pub fn into_globals(self) -> impl Iterator<Item = String> {
        self.globals.into_iter()
    }
}

impl Scope for CollectGlobalsProcessor {
    fn push(&mut self) {
        self.scopes.push(HashSet::new())
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn insert(&mut self, identifier: &mut String) {
        self.add(identifier);
    }

    fn insert_self(&mut self) {
        self.add("self");
    }

    fn insert_local(&mut self, identifier: &mut String, _value: Option<&mut Expression>) {
        self.add(identifier);
    }

    fn insert_local_function(&mut self, function: &mut FunctionAssignment) {
        self.add(function.get_name());
    }
}

impl NodeProcessor for CollectGlobalsProcessor {
    fn process_variable_expression(&mut self, variable: &mut Identifier) {
        if !self.is_declared(variable.get_name()) {
            self.globals.insert(variable.get_name().clone());
        }
    }

    fn process_type_field(&mut self, type_field: &mut TypeField) {
        let namespace = type_field.get_namespace().get_name();
        if !self.is_declared(namespace) {
            self.globals.insert(namespace.clone());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        process::{NodeVisitor, ScopeVisitor},
        Parser,
    };

    use super::*;

    fn extract_globals(code: &str) -> Vec<String> {
        let mut processor = CollectGlobalsProcessor::new();

        let mut block = Parser::default()
            .parse(code)
            .expect("expected code should parse");

        ScopeVisitor::visit_block(&mut block, &mut processor);

        let mut globals = processor.into_globals().collect::<Vec<_>>();
        globals.sort();
        globals
    }

    #[test]
    fn catch_global_variable_in_module_scope() {
        insta::assert_debug_snapshot!(extract_globals(r#"
local g = game
return g
        "#), @r###"
        [
            "game",
        ]
        "###);
    }

    #[test]
    fn catch_global_variable_within_function_scope() {
        insta::assert_debug_snapshot!(extract_globals(r#"
local function example()
    return unpack(game:GetService("Players"):GetPlayers())
end

return { example = example }
        "#), @r###"
        [
            "game",
            "unpack",
        ]
        "###);
    }
}
