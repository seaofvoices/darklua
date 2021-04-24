#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LUXNamespacedName {
    namespace: String,
    member: String,
}

impl LUXNamespacedName {
    #[inline]
    pub fn get_namespace(&self) -> &String {
        &self.namespace
    }

    #[inline]
    pub fn get_member(&self) -> &String {
        &self.member
    }
}

impl From<(String, String)> for LUXNamespacedName {
    fn from((namespace, member): (String, String)) -> Self {
        Self {
            namespace,
            member,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LUXElementName {
    Identifier(String),
    NamespacedName(LUXNamespacedName),
    Members(String, Vec<String>),
}

impl From<String> for LUXElementName {
    fn from(identifier: String) -> Self {
        Self::Identifier(identifier)
    }
}

impl From<LUXNamespacedName> for LUXElementName {
    fn from(name: LUXNamespacedName) -> Self {
        Self::NamespacedName(name)
    }
}

impl From<(String, Vec<String>)> for LUXElementName {
    fn from((root_identifier, members): (String, Vec<String>)) -> Self {
        Self::Members(root_identifier, members)
    }
}
