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
        Self {
            entries,
        }
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
    pub fn mutate_entries(&mut self) -> &mut Vec<TableEntry> {
        &mut self.entries
    }
}

impl Default for TableExpression {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
