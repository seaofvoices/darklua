use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalAssignStatement {
    variables: Vec<String>,
    values: Vec<Expression>,
}

impl LocalAssignStatement {
    pub fn new(variables: Vec<String>, values: Vec<Expression>) -> Self {
        Self {
            variables,
            values,
        }
    }

    pub fn from_variable<S: Into<String>>(variable: S) -> Self {
        Self {
            variables: vec![variable.into()],
            values: Vec::new(),
        }
    }

    pub fn with_variable<S: Into<String>>(mut self, variable: S) -> Self {
        self.variables.push(variable.into());
        self
    }

    pub fn with_value(mut self, value: Expression) -> Self {
        self.values.push(value);
        self
    }

    pub fn into_assignments(self) -> (Vec<String>, Vec<Expression>) {
        (self.variables, self.values)
    }

    pub fn append_assignment<S: Into<String>>(&mut self, variable: S, value: Expression) {
        self.variables.push(variable.into());
        self.values.push(value);
    }

    pub fn for_each_assignment<F>(&mut self, mut callback: F)
        where F: FnMut(&mut String, Option<&mut Expression>)
    {
        let mut values = self.values.iter_mut();
        self.variables.iter_mut()
            .for_each(|variable| callback(variable, values.next()));
    }

    #[inline]
    pub fn get_variables(&self) -> &Vec<String> {
        &self.variables
    }

    #[inline]
    pub fn mutate_variables(&mut self) -> &mut Vec<String> {
        &mut self.variables
    }

    #[inline]
    pub fn get_values(&self) -> &Vec<Expression> {
        &self.values
    }

    #[inline]
    pub fn mutate_values(&mut self) -> &mut Vec<Expression> {
        &mut self.values
    }

    #[inline]
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }
}

impl ToLua for LocalAssignStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("local");

        generator.for_each_and_between(
            &self.variables,
            |generator, variable| generator.push_str(variable),
            |generator| generator.push_char(','),
        );

        if self.values.len() > 0 {
            generator.push_char('=');
        };

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
    fn generate_() {
        let output = LocalAssignStatement::from_variable("var")
            .with_value(Expression::False)
            .to_lua_string();

        assert_eq!(output, "local var=false");
    }
}
