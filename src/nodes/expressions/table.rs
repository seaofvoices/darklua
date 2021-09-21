use crate::nodes::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntry {
    Field(String, Expression),
    Index(Expression, Expression),
    Value(Expression),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableExpression {
    entries: Vec<TableEntry>,
}

impl TableExpression {
    pub fn new(entries: Vec<TableEntry>) -> Self {
        Self { entries }
    }

    #[inline]
    pub fn get_entries(&self) -> &Vec<TableEntry> {
        &self.entries
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn mutate_entries(&mut self) -> &mut Vec<TableEntry> {
        &mut self.entries
    }

    pub fn append_field<S: Into<String>, E: Into<Expression>>(mut self, key: S, value: E) -> Self {
        self.entries
            .push(TableEntry::Field(key.into(), value.into()));
        self
    }

    pub fn append_index<T: Into<Expression>, U: Into<Expression>>(
        mut self,
        key: T,
        value: U,
    ) -> Self {
        self.entries
            .push(TableEntry::Index(key.into(), value.into()));
        self
    }

    pub fn append_array_value<E: Into<Expression>>(mut self, value: E) -> Self {
        self.entries.push(TableEntry::Value(value.into()));
        self
    }
}

impl Default for TableExpression {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
