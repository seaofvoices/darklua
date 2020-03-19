use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntry {
    Field(String, Expression),
    Index(Expression, Expression),
    Value(Expression),
}

impl ToLua for TableEntry {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Field(identifier, expression) => {
                generator.push_str(&identifier);
                generator.push_char('=');
                expression.to_lua(generator);
            }
            Self::Index(key, value) => {
                generator.push_char('[');
                key.to_lua(generator);
                generator.push_str("]=");
                value.to_lua(generator);
            }
            Self::Value(value) => value.to_lua(generator),
        }
    }
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

    pub fn get_entries(&self) -> &Vec<TableEntry> {
        &self.entries
    }

    pub fn mutate_entries(&mut self) -> &mut Vec<TableEntry> {
        &mut self.entries
    }
}

impl Default for TableExpression {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl ToLua for TableExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_char('{');

        let last_index = self.entries.len().checked_sub(1).unwrap_or(0);

        self.entries.iter().enumerate().for_each(|(index, entry)| {
            entry.to_lua(generator);

            if index != last_index {
                generator.push_char(',');
            }
        });
        generator.push_char('}');
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_empty_table_expression() {
        let output = TableExpression::default().to_lua_string();

        assert_eq!(output, "{}");
    }

    #[test]
    fn generate_list_table_expression_with_single_value() {
        let output = TableExpression::new(vec![
            TableEntry::Value(Expression::True),
        ]).to_lua_string();

        assert_eq!(output, "{true}");
    }

    #[test]
    fn generate_list_table_expression_with_two_values() {
        let output = TableExpression::new(vec![
            TableEntry::Value(Expression::True),
            TableEntry::Value(Expression::False),
        ]).to_lua_string();

        assert_eq!(output, "{true,false}");
    }

    #[test]
    fn generate_table_expression_with_field_value() {
        let output = TableExpression::new(vec![
            TableEntry::Field("field".to_owned(), Expression::True),
        ]).to_lua_string();

        assert_eq!(output, "{field=true}");
    }

    #[test]
    fn generate_table_expression_with_index_value() {
        let output = TableExpression::new(vec![
            TableEntry::Index(Expression::False, Expression::True),
        ]).to_lua_string();

        assert_eq!(output, "{[false]=true}");
    }

    #[test]
    fn generate_mixed_table_expression() {
        let output = TableExpression::new(vec![
            TableEntry::Value(Expression::True),
            TableEntry::Field("field".to_owned(), Expression::True),
            TableEntry::Index(Expression::False, Expression::True),
        ]).to_lua_string();

        assert_eq!(output, "{true,field=true,[false]=true}");
    }
}
