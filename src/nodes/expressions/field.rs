use crate::nodes::Prefix;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldExpression {
    prefix: Prefix,
    field: String,
}

impl FieldExpression {
    pub fn new<IntoPrefix: Into<Prefix>, S: Into<String>>(prefix: IntoPrefix, field: S) -> Self {
        Self {
            prefix: prefix.into(),
            field: field.into(),
        }
    }

    #[inline]
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[inline]
    pub fn get_field(&self) -> &String {
        &self.field
    }

    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }
}
