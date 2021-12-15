#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionEffect {
    mutated_identifiers: Vec<Vec<String>>,
}

impl ExecutionEffect {
    pub fn add<S: Into<String>>(&mut self, identifier: S) {
        if let Some(identifiers) = self.mutated_identifiers.last_mut() {
            identifiers.push(identifier.into());
        }
    }

    pub fn enable(&mut self) {
        self.mutated_identifiers.push(Vec::new());
    }

    pub fn disable(&mut self) -> impl Iterator<Item = String> {
        if let Some(identifiers) = self.mutated_identifiers.pop() {
            identifiers.into_iter()
        } else {
            Vec::new().into_iter()
        }
    }
}
