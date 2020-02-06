use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{
    Expression,
    FieldExpression,
    IndexExpression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Identifier(String),
    Field(Box<FieldExpression>),
    Index(Box<IndexExpression>),
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self::Identifier(name.into())
    }
}

impl ToLua for Variable {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Identifier(name) => generator.push_str(name),
            Self::Field(field) => field.to_lua(generator),
            Self::Index(index) => index.to_lua(generator),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignStatement {
    variables: Vec<Variable>,
    values: Vec<Expression>,
}

impl AssignStatement {
    pub fn new(variables: Vec<Variable>, values: Vec<Expression>) -> Self {
        Self {
            variables,
            values,
        }
    }

    pub fn mutate_variables(&mut self) -> &mut Vec<Variable> {
        &mut self.variables
    }

    pub fn mutate_values(&mut self) -> &mut Vec<Expression> {
        &mut self.values
    }
}

impl ToLua for AssignStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.for_each_and_between(
            &self.variables,
            |generator, variable| variable.to_lua(generator),
            |generator| generator.push_char(','),
        );
        generator.push_char('=');
        generator.for_each_and_between(
            &self.values,
            |generator, expression| expression.to_lua(generator),
            |generator| generator.push_char(','),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_variable_with_one_value() {
        let output = AssignStatement::new(
            vec![Variable::new("var")],
            vec![Expression::False],
        ).to_lua_string();

        assert_eq!(output, "var=false");
    }

    #[test]
    fn generate_two_variables_with_one_value() {
        let output = AssignStatement::new(
            vec![Variable::new("foo"), Variable::new("var")],
            vec![Expression::False],
        ).to_lua_string();

        assert_eq!(output, "foo,var=false");
    }

    #[test]
    fn generate_two_variables_with_two_values() {
        let output = AssignStatement::new(
            vec![Variable::new("foo"), Variable::new("var")],
            vec![Expression::Nil, Expression::False],
        ).to_lua_string();

        assert_eq!(output, "foo,var=nil,false");
    }
}
