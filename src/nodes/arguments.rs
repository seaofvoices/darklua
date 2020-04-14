use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{
    Expression,
    StringExpression,
    TableExpression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arguments {
    Tuple(Vec<Expression>),
    String(StringExpression),
    Table(TableExpression),
}

impl Arguments {
    pub fn to_expressions(self) -> Vec<Expression> {
        match self {
            Self::Tuple(expressions) => expressions,
            Self::String(string) => vec![string.into()],
            Self::Table(table) => vec![table.into()],
        }
    }

    pub fn append_argument<T: Into<Expression>>(self, argument: T) -> Self {
        let mut expressions = self.to_expressions();
        expressions.push(argument.into());
        Arguments::Tuple(expressions)
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments::Tuple(Vec::new())
    }
}

impl ToLua for Arguments {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::String(string) => string.to_lua(generator),
            Self::Table(table) => table.to_lua(generator),
            Self::Tuple(expressions) => {
                generator.merge_char('(');

                let last_index = expressions.len().checked_sub(1).unwrap_or(0);
                expressions.iter().enumerate()
                    .for_each(|(index, expression)| {
                        expression.to_lua(generator);

                        if index != last_index {
                            generator.push_char(',');
                        }
                    });

                generator.push_char(')');
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_tuple_does_not_break_on_new_line() {
        let mut generator = LuaGenerator::new(16);
        generator.push_str("123");
        generator.push_str("functionName");

        Arguments::Tuple(Vec::new()).to_lua(&mut generator);

        assert_eq!(generator.into_string(), "123\nfunctionName()");
    }

    mod snapshot {
        use super::*;

        use insta::assert_snapshot;

        fn get_expression() -> Expression {
            Expression::Identifier("foo".to_owned()).into()
        }

        #[test]
        fn empty_tuple() {
            assert_snapshot!("empty_tuple", Arguments::Tuple(vec![]).to_lua_string());
        }

        #[test]
        fn single_argument() {
            let arguments = Arguments::Tuple(vec![get_expression()]);

            assert_snapshot!("single_argument", arguments.to_lua_string());
        }

        #[test]
        fn two_arguments() {
            let arguments = Arguments::Tuple(vec![
                get_expression(),
                get_expression(),
            ]);

            assert_snapshot!("two_arguments", arguments.to_lua_string());
        }
    }
}
