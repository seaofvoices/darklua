use crate::nodes::{
    Expression,
    FieldExpression,
    FunctionCall,
    IndexExpression,
};
use crate::lua_generator::{LuaGenerator, ToLua};


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prefix {
    Call(FunctionCall),
    Field(Box<FieldExpression>),
    Identifier(String),
    Index(Box<IndexExpression>),
    Parenthese(Expression),
}

impl Prefix {
    pub fn from_name<S: Into<String>>(name: S) -> Self {
        Self::Identifier(name.into())
    }
}

impl ToLua for Prefix {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Call(call) => call.to_lua(generator),
            Self::Field(field) => field.to_lua(generator),
            Self::Identifier(identifier) => generator.push_str(identifier),
            Self::Index(index) => index.to_lua(generator),
            Self::Parenthese(expression) => {
                generator.push_char('(');
                expression.to_lua(generator);
                generator.push_char(')');
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_identifier() {
        let output = Prefix::from_name("foo").to_lua_string();

        assert_eq!(output, "foo");
    }

    #[test]
    fn generate_parenthese() {
        let output = Prefix::Parenthese(Prefix::from_name("foo").into()).to_lua_string();

        assert_eq!(output, "(foo)");
    }
}
