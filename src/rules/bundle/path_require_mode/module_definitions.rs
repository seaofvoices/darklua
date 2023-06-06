use std::path::Path;

use crate::frontend::DarkluaResult;
use crate::nodes::{
    AssignStatement, Block, DoStatement, Expression, FieldExpression, Identifier, LastStatement,
    LocalAssignStatement, TableExpression,
};
use crate::process::utils::{generate_identifier, identifier_permutator, CharPermutator};
use crate::DarkluaError;

use super::RequiredResource;

#[derive(Debug)]
pub(crate) struct BuildModuleDefinitions<'a> {
    modules_identifier: &'a str,
    module_definitions: Vec<Block>,
    module_name_permutator: CharPermutator,
}

impl<'a> BuildModuleDefinitions<'a> {
    pub(crate) fn new(modules_identifier: &'a str) -> Self {
        Self {
            modules_identifier,
            module_definitions: Vec::new(),
            module_name_permutator: identifier_permutator(),
        }
    }

    pub(crate) fn build_module_from_resource(
        &mut self,
        required_resource: RequiredResource,
        require_path: &Path,
    ) -> DarkluaResult<Expression> {
        let (module_name, block) = match required_resource {
            RequiredResource::Block(mut block) => {
                let module_name = if let Some(LastStatement::Return(return_statement)) =
                    block.take_last_statement()
                {
                    if return_statement.len() != 1 {
                        return Err(DarkluaError::custom(format!(
                            "invalid Lua module at `{}`: module must return exactly one value",
                            require_path.display()
                        )));
                    }

                    let return_value = return_statement.into_iter_expressions().next().unwrap();
                    let module_name = generate_identifier(&mut self.module_name_permutator);

                    block.push_statement(AssignStatement::from_variable(
                        FieldExpression::new(
                            Identifier::from(self.modules_identifier),
                            module_name.clone(),
                        ),
                        return_value,
                    ));

                    module_name
                } else {
                    return Err(DarkluaError::custom(format!(
                        "invalid Lua module at `{}`: module must end with a return statement",
                        require_path.display()
                    )));
                };

                (module_name, block)
            }
            RequiredResource::Expression(expression) => {
                let module_name = generate_identifier(&mut self.module_name_permutator);
                let block = Block::default().with_statement(AssignStatement::from_variable(
                    FieldExpression::new(
                        Identifier::from(self.modules_identifier),
                        module_name.clone(),
                    ),
                    expression,
                ));

                (module_name, block)
            }
        };

        self.module_definitions.push(block);

        Ok(FieldExpression::new(Identifier::from(self.modules_identifier), module_name).into())
    }

    pub(crate) fn apply(mut self, block: &mut Block) {
        if self.module_definitions.is_empty() {
            return;
        }
        for module_block in self.module_definitions.drain(..).rev() {
            block.insert_statement(0, DoStatement::new(module_block));
        }
        block.insert_statement(
            0,
            LocalAssignStatement::from_variable(self.modules_identifier)
                .with_value(TableExpression::default()),
        );
    }
}
