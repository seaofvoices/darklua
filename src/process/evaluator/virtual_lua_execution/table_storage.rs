use crate::process::TableValue;

pub type TableId = usize;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStorage {
    tables: Vec<TableValue>,
}

impl TableStorage {
    pub fn insert(&mut self, table: TableValue) -> TableId {
        let id = self.tables.len();
        self.tables.push(table);
        id
    }

    pub fn mutate(&mut self, id: TableId) -> Option<&mut TableValue> {
        self.tables.get_mut(id)
    }

    pub fn get(&self, id: TableId) -> Option<&TableValue> {
        self.tables.get(id)
    }
}
