use crate::nodes::Block;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Feature {
    ShallowMerge,
}

/// The intent of this struct is to hold data shared across all rules applied to a file.
#[derive(Debug, Clone, Default)]
pub struct Context {
    features: HashMap<Feature, String>,
}

impl Context {
    pub fn pre_process_block(&mut self, block: &Block) {

    }

    pub fn post_process_block(&self, block: &mut Block) {

    }

    /// Returns an identifier for a shallow merge function
    pub fn request_shallow_merge_identifier(&mut self) -> String {
        if let Some(identifier) = self.features.get(&Feature::ShallowMerge) {
            identifier.to_owned()
        } else {
            let id = "_DARKLUA_SHALLOW_MERGE".to_owned();
            self.features.insert(Feature::ShallowMerge, id.clone());
            id
        }
    }
}
