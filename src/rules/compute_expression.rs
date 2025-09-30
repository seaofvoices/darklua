use crate::nodes::{
    BinaryOperator, Block, Expression, FunctionExpression, FunctionStatement, GenericForStatement,
    LocalFunctionStatement, NumericForStatement, Prefix, RepeatStatement, Variable, WhileStatement,
};
use crate::process::{
    Evaluator, EvaluatorStorage, FunctionValue, LuaValue, NativeFunction, NodeProcessor,
    NodeVisitor, Scope, ScopeVisitor, TableValue,
};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

#[derive(Debug, Default)]
struct Computer {
    evaluator: Evaluator,
    storage: EvaluatorStorage,
}

impl Computer {
    fn define_base_environment(&mut self) {
        let mut math_table = TableValue::new().with_pure_metamethods();
        math_table.insert(LuaValue::from("abs"), NativeFunction::math_abs().into());
        math_table.insert(LuaValue::from("cos"), NativeFunction::math_cos().into());
        math_table.insert(LuaValue::from("sin"), NativeFunction::math_sin().into());
        math_table.insert(LuaValue::from("tan"), NativeFunction::math_tan().into());
        math_table.insert(LuaValue::from("sqrt"), NativeFunction::math_sqrt().into());
        math_table.insert(LuaValue::from("pow"), NativeFunction::math_pow().into());
        math_table.insert(LuaValue::from("exp"), NativeFunction::math_exp().into());
        math_table.insert(LuaValue::from("deg"), NativeFunction::math_deg().into());
        math_table.insert(LuaValue::from("rad"), NativeFunction::math_rad().into());
        math_table.insert(LuaValue::from("sign"), NativeFunction::math_sign().into());

        math_table.insert(LuaValue::from("huge"), LuaValue::Number(f64::INFINITY));
        math_table.insert(LuaValue::from("pi"), LuaValue::Number(std::f64::consts::PI));

        self.storage
            .declare_identifier("math", Some(self.storage.create_table(math_table)));
    }

    fn replace_with(&mut self, expression: &Expression) -> Option<Expression> {
        match expression {
            Expression::Binary(binary) => {
                if !self.has_effects(expression) {
                    self.compute(expression).to_expression().or_else(|| {
                        match binary.operator() {
                            BinaryOperator::And => {
                                self.compute(binary.left()).is_truthy().map(|is_truthy| {
                                    if is_truthy {
                                        binary.right().clone()
                                    } else {
                                        binary.left().clone()
                                    }
                                })
                            }
                            BinaryOperator::Or => {
                                self.compute(binary.left()).is_truthy().map(|is_truthy| {
                                    if is_truthy {
                                        binary.left().clone()
                                    } else {
                                        binary.right().clone()
                                    }
                                })
                            }
                            _ => None,
                        }
                        .map(|mut expression| {
                            self.process_expression(&mut expression);
                            expression
                        })
                    })
                } else {
                    match binary.operator() {
                        BinaryOperator::And => {
                            if !self.has_effects(binary.left()) {
                                self.compute(binary.left()).is_truthy().map(|is_truthy| {
                                    if is_truthy {
                                        binary.right().clone()
                                    } else {
                                        binary.left().clone()
                                    }
                                })
                            } else {
                                None
                            }
                        }
                        BinaryOperator::Or => {
                            if !self.has_effects(binary.left()) {
                                self.compute(binary.left()).is_truthy().map(|is_truthy| {
                                    if is_truthy {
                                        binary.left().clone()
                                    } else {
                                        binary.right().clone()
                                    }
                                })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
            }
            Expression::Identifier(_)
            | Expression::Call(_)
            | Expression::Unary(_)
            | Expression::If(_) => {
                if !self.has_effects(expression) {
                    println!("compute_expression: {:?}", expression);
                    self.compute(expression).to_expression()
                } else {
                    println!("skip because of side effects: {:?}", expression);
                    None
                }
            }
            _ => None,
        }
    }

    fn compute(&self, expression: &Expression) -> LuaValue {
        self.evaluator.evaluate_internal(expression, &self.storage)
    }

    fn has_effects(&self, expression: &Expression) -> bool {
        self.evaluator
            .has_side_effects_internal(expression, &self.storage)
    }

    fn process_variable_prefix(&mut self, prefix: &Prefix) {
        let mut current = prefix;

        loop {
            match current {
                Prefix::Call(function_call) => {
                    current = function_call.get_prefix();
                }
                Prefix::Field(field) => {
                    current = field.get_prefix();
                }
                Prefix::Identifier(identifier) => {
                    self.storage.mark_mutated(identifier.get_name());
                    break;
                }
                Prefix::Index(index) => {
                    current = index.get_prefix();
                }
                Prefix::Parenthese(parenthese) => {
                    // todo
                }
            }
        }
    }
}

impl NodeProcessor for Computer {
    fn process_expression(&mut self, expression: &mut Expression) {
        if let Some(replace_with) = self.replace_with(expression) {
            *expression = replace_with;
        }
    }

    fn process_prefix_expression(&mut self, prefix: &mut Prefix) {
        match prefix {
            Prefix::Identifier(identifier) => {
                self.storage.mark_mutated(identifier.get_name());
            }
            Prefix::Call(_) | Prefix::Field(_) | Prefix::Index(_) | Prefix::Parenthese(_) => {}
        }
    }

    fn process_variable(&mut self, variable: &mut Variable) {
        match variable {
            Variable::Identifier(identifier) => {
                self.storage.mark_mutated(identifier.get_name());
            }
            Variable::Field(field) => {
                self.process_variable_prefix(field.get_prefix());
            }
            Variable::Index(index) => {
                self.process_variable_prefix(index.get_prefix());
            }
        }
    }

    fn process_repeat_statement(&mut self, _statement: &mut RepeatStatement) {
        // todo
    }

    fn process_while_statement(&mut self, _statement: &mut WhileStatement) {
        // todo
    }

    fn process_generic_for_statement(&mut self, _statement: &mut GenericForStatement) {
        // todo
    }

    fn process_numeric_for_statement(&mut self, _statement: &mut NumericForStatement) {
        // todo
    }

    fn process_function_statement(&mut self, _statement: &mut FunctionStatement) {
        // todo
    }

    fn process_local_function_statement(&mut self, _statement: &mut LocalFunctionStatement) {
        // todo
    }

    fn process_function_expression(&mut self, _statement: &mut FunctionExpression) {
        // todo
    }
}

impl Scope for Computer {
    fn push(&mut self) {
        self.storage.push_scope();
    }

    fn pop(&mut self) {
        self.storage.pop_scope();
    }

    fn insert(&mut self, identifier: &mut String) {
        self.storage.declare_identifier(identifier, None);
    }

    fn insert_self(&mut self) {
        self.storage
            .declare_identifier("self", Some(LuaValue::Unknown));
    }

    fn insert_local(&mut self, identifier: &mut String, value: Option<&mut Expression>) {
        self.storage.declare_identifier(
            identifier,
            value
                .map(|value| self.compute(value))
                .or_else(|| Some(LuaValue::Unknown)),
        );
    }

    fn insert_local_function(&mut self, function: &mut LocalFunctionStatement) {
        self.storage.declare_identifier(
            function.get_name(),
            Some(LuaValue::Function(FunctionValue::new_lua())),
        );
    }
}

pub const COMPUTE_EXPRESSIONS_RULE_NAME: &str = "compute_expression";

/// A rule that compute expressions that do not have any side-effects.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ComputeExpression {}

impl FlawlessRule for ComputeExpression {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Computer::default();
        // todo: use a configuration value to define the base environment
        processor.define_base_environment();
        ScopeVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ComputeExpression {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        COMPUTE_EXPRESSIONS_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> ComputeExpression {
        ComputeExpression::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_compute_expression", rule);
    }
}
