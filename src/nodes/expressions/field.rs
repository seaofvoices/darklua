use crate::nodes::Prefix;
use crate::lua_generator::{LuaGenerator, ToLua};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldExpression {
    prefix: Prefix,
    field: String,
}

impl FieldExpression {
    pub fn new<S: Into<String>>(prefix: Prefix, field: S) -> Self {
        Self {
            prefix,
            field: field.into(),
        }
    }

    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }
}

impl ToLua for FieldExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        self.prefix.to_lua(generator);
        generator.push_char('.');
        generator.push_str(&self.field);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_identifier() {
        let output = FieldExpression::new(Prefix::from_name("foo"), "bar").to_lua_string();

        assert_eq!(output, "foo.bar");
    }
}
