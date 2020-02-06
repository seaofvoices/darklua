use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastStatement {
    Break,
    Return(Vec<Expression>),
}

impl ToLua for LastStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Break => generator.push_str("break"),
            Self::Return(expressions) => {
                generator.push_str("return");

                generator.for_each_and_between(
                    expressions,
                    |generator, expression| expression.to_lua(generator),
                    |generator| generator.push_char(',')
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_break_statement() {
        let output = LastStatement::Break.to_lua_string();

        assert_eq!(output, "break");
    }

    #[test]
    fn generate_return_statement_without_values() {
        let output = LastStatement::Return(vec![]).to_lua_string();

        assert_eq!(output, "return");
    }

    #[test]
    fn generate_return_statement_with_one_expression() {
        let expressions = vec![Expression::Identifier("var".to_owned())];
        let output = LastStatement::Return(expressions).to_lua_string();

        assert_eq!(output, "return var");
    }

    #[test]
    fn generate_return_statement_with_two_expressions() {
        let var = Expression::Identifier("var".to_owned());
        let expressions = vec![var.clone(), var];
        let output = LastStatement::Return(expressions).to_lua_string();

        assert_eq!(output, "return var,var");
    }
}
