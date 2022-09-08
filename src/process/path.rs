use std::{borrow::Borrow, fmt, ops::Deref, str::FromStr};

use crate::nodes::{
    AnyExpressionRef, AnyExpressionRefMut, AnyNodeRef, AnyNodeRefMut, AnyStatementRef,
    AnyStatementRefMut, Arguments, Block, Expression, FieldExpression, FunctionCall,
    IndexExpression, LastStatement, ParentheseExpression, Prefix, Statement, TableExpression,
    Variable,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Component {
    Block(usize),
    Expression(usize),
    Statement(usize),
}

pub type NodePathSlice = [Component];

pub trait NodePath {
    fn parent(&self) -> Option<&NodePathSlice>;
    fn common_ancestor(&self, other: &Self) -> &NodePathSlice;
    fn is_statement_path(&self) -> bool;
    fn last_statement(&self) -> Option<usize>;
    fn find_first_statement_ancestor(&self) -> Option<&NodePathSlice>;

    fn to_path_buf(&self) -> NodePathBuf;
    fn to_string(&self) -> String;

    fn join_statement(&self, index: usize) -> NodePathBuf;
    fn join_expression(&self, index: usize) -> NodePathBuf;
    fn join_block(&self, index: usize) -> NodePathBuf;

    fn resolve<'a>(&self, block: &'a Block) -> Option<AnyNodeRef<'a>>;
    fn resolve_mut<'a>(&self, block: &'a mut Block) -> Option<AnyNodeRefMut<'a>>;

    fn resolve_statement<'a>(&self, block: &'a Block) -> Option<AnyStatementRef<'a>>;
    fn resolve_statement_mut<'a>(&self, block: &'a mut Block) -> Option<AnyStatementRefMut<'a>>;

    fn resolve_expression<'a>(&self, block: &'a Block) -> Option<AnyExpressionRef<'a>>;
    fn resolve_expression_mut<'a>(&self, block: &'a mut Block) -> Option<AnyExpressionRefMut<'a>>;

    fn resolve_block<'a>(&self, block: &'a Block) -> Option<&'a Block>;
    fn resolve_block_mut<'a>(&self, block: &'a mut Block) -> Option<&'a mut Block>;
}

impl NodePath for NodePathSlice {
    fn parent(&self) -> Option<&NodePathSlice> {
        if self.len() == 0 {
            None
        } else {
            self.get(0..self.len() - 1)
        }
    }

    fn common_ancestor(&self, other: &Self) -> &NodePathSlice {
        let slice_end = self
            .into_iter()
            .zip(other)
            .enumerate()
            .find(|(_, (component, other_component))| component != other_component)
            .map(|(index, _)| index)
            .unwrap_or(0);

        self.get(0..slice_end).unwrap_or(&[])
    }

    fn is_statement_path(&self) -> bool {
        self.last()
            .map(|last| matches!(last, Component::Statement(_)))
            .unwrap_or(false)
    }

    fn last_statement(&self) -> Option<usize> {
        self.last().map(|last| match last {
            Component::Block(_) => todo!(),
            Component::Expression(_) => todo!(),
            Component::Statement(index) => *index,
        })
    }

    fn find_first_statement_ancestor(&self) -> Option<&NodePathSlice> {
        let mut ancestor = self;

        while !ancestor.is_statement_path() {
            ancestor = ancestor.parent()?;
        }

        Some(ancestor)
    }

    fn to_string(&self) -> String {
        self.iter()
            .map(|component| match component {
                Component::Block(index) => format!("{}{}", index, BLOCK_DELIMITER),
                Component::Expression(index) => format!("{}{}", index, EXPRESSION_DELIMITER),
                Component::Statement(index) => format!("{}{}", index, STATEMENT_DELIMITER),
            })
            .collect()
    }

    #[inline]
    fn to_path_buf(&self) -> NodePathBuf {
        NodePathBuf::from(self)
    }

    #[inline]
    fn join_statement(&self, index: usize) -> NodePathBuf {
        NodePathBuf::new(self).with_statement(index)
    }

    #[inline]
    fn join_expression(&self, index: usize) -> NodePathBuf {
        NodePathBuf::new(self).with_expression(index)
    }

    #[inline]
    fn join_block(&self, index: usize) -> NodePathBuf {
        NodePathBuf::new(self).with_block(index)
    }

    fn resolve<'a>(&self, block: &'a Block) -> Option<AnyNodeRef<'a>> {
        let mut current = AnyNodeRef::AnyBlock(&block);

        for current_component in self.iter() {
            current = match current_component {
                Component::Block(index) => resolve_block(current, *index)?.into(),
                Component::Expression(index) => {
                    let next_expression = resolve_expression(current, *index)?;
                    AnyNodeRef::from(next_expression)
                }
                Component::Statement(index) => {
                    let next_statement = resolve_statement(current, *index)?;
                    AnyNodeRef::from(next_statement)
                }
            }
        }

        Some(current)
    }

    fn resolve_mut<'a>(&self, block: &'a mut Block) -> Option<AnyNodeRefMut<'a>> {
        let mut current = AnyNodeRefMut::AnyBlock(block);

        for current_component in self.iter() {
            current = match current_component {
                Component::Block(index) => resolve_block_mut(current, *index)?.into(),
                Component::Expression(index) => {
                    let next_expression = resolve_expression_mut(current, *index)?;
                    AnyNodeRefMut::from(next_expression)
                }
                Component::Statement(index) => {
                    let next_statement = resolve_statement_mut(current, *index)?;
                    AnyNodeRefMut::from(next_statement)
                }
            }
        }

        Some(current)
    }

    fn resolve_statement<'a>(&self, block: &'a Block) -> Option<AnyStatementRef<'a>> {
        match self.resolve(block)? {
            AnyNodeRef::AnyStatement(statement) => Some(statement),
            AnyNodeRef::AnyBlock(_) | AnyNodeRef::AnyExpression(_) => None,
        }
    }

    fn resolve_statement_mut<'a>(&self, block: &'a mut Block) -> Option<AnyStatementRefMut<'a>> {
        match self.resolve_mut(block)? {
            AnyNodeRefMut::AnyStatement(statement) => Some(statement),
            AnyNodeRefMut::AnyBlock(_) | AnyNodeRefMut::AnyExpression(_) => None,
        }
    }

    fn resolve_expression<'a>(&self, block: &'a Block) -> Option<AnyExpressionRef<'a>> {
        match self.resolve(block)? {
            AnyNodeRef::AnyBlock(_) | AnyNodeRef::AnyStatement(_) => None,
            AnyNodeRef::AnyExpression(expression) => Some(expression),
        }
    }

    fn resolve_expression_mut<'a>(&self, block: &'a mut Block) -> Option<AnyExpressionRefMut<'a>> {
        match self.resolve_mut(block)? {
            AnyNodeRefMut::AnyBlock(_) | AnyNodeRefMut::AnyStatement(_) => None,
            AnyNodeRefMut::AnyExpression(expression) => Some(expression),
        }
    }

    fn resolve_block<'a>(&self, block: &'a Block) -> Option<&'a Block> {
        match self.resolve(block)? {
            AnyNodeRef::AnyBlock(block) => Some(block),
            AnyNodeRef::AnyStatement(any_statement) => match any_statement {
                AnyStatementRef::Statement(statement) => Some(match statement {
                    Statement::Do(do_statement) => do_statement.get_block(),
                    Statement::Function(function) => function.get_block(),
                    Statement::GenericFor(generic_for) => generic_for.get_block(),
                    Statement::LocalFunction(function) => function.get_block(),
                    Statement::NumericFor(numeric_for) => numeric_for.get_block(),
                    Statement::Repeat(repeat) => repeat.get_block(),
                    Statement::While(while_statement) => while_statement.get_block(),
                    Statement::Assign(_)
                    | Statement::Call(_)
                    | Statement::CompoundAssign(_)
                    | Statement::If(_)
                    | Statement::LocalAssign(_) => return None,
                }),
                AnyStatementRef::LastStatement(_) => None,
            },
            AnyNodeRef::AnyExpression(_) => None,
        }
    }

    fn resolve_block_mut<'a>(&self, block: &'a mut Block) -> Option<&'a mut Block> {
        match self.resolve_mut(block)? {
            AnyNodeRefMut::AnyBlock(block) => Some(block),
            AnyNodeRefMut::AnyStatement(any_statement) => match any_statement {
                AnyStatementRefMut::Statement(statement) => Some(match statement {
                    Statement::Do(do_statement) => do_statement.mutate_block(),
                    Statement::Function(function) => function.mutate_block(),
                    Statement::GenericFor(generic_for) => generic_for.mutate_block(),
                    Statement::LocalFunction(function) => function.mutate_block(),
                    Statement::NumericFor(numeric_for) => numeric_for.mutate_block(),
                    Statement::Repeat(repeat) => repeat.mutate_block(),
                    Statement::While(while_statement) => while_statement.mutate_block(),
                    Statement::Assign(_)
                    | Statement::Call(_)
                    | Statement::CompoundAssign(_)
                    | Statement::If(_)
                    | Statement::LocalAssign(_) => return None,
                }),
                AnyStatementRefMut::LastStatement(_) => None,
            },
            AnyNodeRefMut::AnyExpression(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodePathBuf {
    components: Vec<Component>,
}

impl Borrow<NodePathSlice> for NodePathBuf {
    #[inline]
    fn borrow(&self) -> &NodePathSlice {
        &self.components
    }
}

impl Deref for NodePathBuf {
    type Target = NodePathSlice;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl PartialEq<NodePathSlice> for NodePathBuf {
    #[inline]
    fn eq(&self, other: &NodePathSlice) -> bool {
        self.components == other
    }
}

impl fmt::Display for NodePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for component in &self.components {
            let (index, delimiter) = match component {
                Component::Block(index) => (index, BLOCK_DELIMITER),
                Component::Expression(index) => (index, EXPRESSION_DELIMITER),
                Component::Statement(index) => (index, STATEMENT_DELIMITER),
            };
            f.write_fmt(format_args!("{}", index))?;
            f.write_str(delimiter)?;
        }
        Ok(())
    }
}

impl NodePathBuf {
    pub fn new(components: impl Into<Vec<Component>>) -> Self {
        Self {
            components: components.into(),
        }
    }

    pub fn statement_index(&self) -> Option<usize> {
        self.components
            .last()
            .and_then(|component| match component {
                Component::Statement(index) => Some(*index),
                Component::Block(_) | Component::Expression(_) => None,
            })
    }

    #[inline]
    pub fn pop_component(&mut self) {
        self.components.pop();
    }

    #[inline]
    pub fn push_statement(&mut self, index: usize) {
        self.components.push(Component::Statement(index));
    }

    #[inline]
    fn push_expression(&mut self, index: usize) {
        self.components.push(Component::Expression(index));
    }

    #[inline]
    fn push_block(&mut self, index: usize) {
        self.components.push(Component::Block(index));
    }

    pub fn with_statement(mut self, index: usize) -> Self {
        self.push_statement(index);
        self
    }

    pub fn with_expression(mut self, index: usize) -> Self {
        self.push_expression(index);
        self
    }

    pub fn with_block(mut self, index: usize) -> Self {
        self.push_block(index);
        self
    }
}

impl From<&NodePathSlice> for NodePathBuf {
    fn from(components: &NodePathSlice) -> Self {
        Self {
            components: components.into_iter().map(Clone::clone).collect(),
        }
    }
}

fn resolve_block<'a>(node: AnyNodeRef<'a>, index: usize) -> Option<&'a Block> {
    if index == 0 {
        let block = match node {
            AnyNodeRef::AnyStatement(any_statement) => match any_statement {
                AnyStatementRef::Statement(statement) => match statement {
                    Statement::Do(do_statement) => do_statement.get_block(),
                    Statement::Function(function) => function.get_block(),
                    Statement::GenericFor(generic_for) => generic_for.get_block(),
                    Statement::If(if_statement) => if_statement.get_block(0)?,
                    Statement::LocalFunction(function) => function.get_block(),
                    Statement::NumericFor(numeric_for) => numeric_for.get_block(),
                    Statement::Repeat(repeat) => repeat.get_block(),
                    Statement::While(while_statement) => while_statement.get_block(),
                    Statement::Assign(_)
                    | Statement::Call(_)
                    | Statement::CompoundAssign(_)
                    | Statement::LocalAssign(_) => return None,
                },
                AnyStatementRef::LastStatement(_) => return None,
            },
            AnyNodeRef::AnyExpression(_) => return None,
            AnyNodeRef::AnyBlock(_) => return None,
        };
        Some(block)
    } else {
        match node {
            AnyNodeRef::AnyStatement(any_statement) => match any_statement {
                AnyStatementRef::Statement(statement) => match statement {
                    Statement::If(if_statement) => if_statement.get_block(index),
                    Statement::Do(_)
                    | Statement::Function(_)
                    | Statement::GenericFor(_)
                    | Statement::LocalFunction(_)
                    | Statement::NumericFor(_)
                    | Statement::Repeat(_)
                    | Statement::While(_)
                    | Statement::Assign(_)
                    | Statement::Call(_)
                    | Statement::CompoundAssign(_)
                    | Statement::LocalAssign(_) => None,
                },
                AnyStatementRef::LastStatement(_) => None,
            },
            AnyNodeRef::AnyExpression(_) => None,
            AnyNodeRef::AnyBlock(_) => None,
        }
    }
}

fn resolve_block_mut<'a>(node: AnyNodeRefMut<'a>, index: usize) -> Option<&'a mut Block> {
    if index == 0 {
        let block = match node {
            AnyNodeRefMut::AnyStatement(any_statement) => match any_statement {
                AnyStatementRefMut::Statement(statement) => match statement {
                    Statement::Do(do_statement) => do_statement.mutate_block(),
                    Statement::Function(function) => function.mutate_block(),
                    Statement::GenericFor(generic_for) => generic_for.mutate_block(),
                    Statement::If(if_statement) => if_statement.mutate_block(0)?,
                    Statement::LocalFunction(function) => function.mutate_block(),
                    Statement::NumericFor(numeric_for) => numeric_for.mutate_block(),
                    Statement::Repeat(repeat) => repeat.mutate_block(),
                    Statement::While(while_statement) => while_statement.mutate_block(),
                    Statement::Assign(_)
                    | Statement::Call(_)
                    | Statement::CompoundAssign(_)
                    | Statement::LocalAssign(_) => return None,
                },
                AnyStatementRefMut::LastStatement(_) => return None,
            },
            AnyNodeRefMut::AnyExpression(_) => return None,
            AnyNodeRefMut::AnyBlock(_) => return None,
        };
        Some(block)
    } else {
        match node {
            AnyNodeRefMut::AnyStatement(any_statement) => match any_statement {
                AnyStatementRefMut::Statement(statement) => match statement {
                    Statement::If(if_statement) => if_statement.mutate_block(index),
                    Statement::Do(_)
                    | Statement::Function(_)
                    | Statement::GenericFor(_)
                    | Statement::LocalFunction(_)
                    | Statement::NumericFor(_)
                    | Statement::Repeat(_)
                    | Statement::While(_)
                    | Statement::Assign(_)
                    | Statement::Call(_)
                    | Statement::CompoundAssign(_)
                    | Statement::LocalAssign(_) => None,
                },
                AnyStatementRefMut::LastStatement(_) => None,
            },
            AnyNodeRefMut::AnyExpression(_) => None,
            AnyNodeRefMut::AnyBlock(_) => None,
        }
    }
}

fn resolve_statement<'a>(node: AnyNodeRef<'a>, index: usize) -> Option<AnyStatementRef<'a>> {
    match node {
        AnyNodeRef::AnyStatement(any_statement) => {
            resolve_statement_from_statement(any_statement, index)
        }
        AnyNodeRef::AnyExpression(any_expression) => {
            resolve_statement_from_expression(any_expression, index)
        }
        AnyNodeRef::AnyBlock(block) => block.get_statement(index),
    }
}

fn resolve_statement_mut<'a>(
    node: AnyNodeRefMut<'a>,
    index: usize,
) -> Option<AnyStatementRefMut<'a>> {
    match node {
        AnyNodeRefMut::AnyStatement(any_statement) => {
            resolve_statement_from_statement_mut(any_statement, index)
        }
        AnyNodeRefMut::AnyExpression(any_expression) => {
            resolve_statement_from_expression_mut(any_expression, index)
        }
        AnyNodeRefMut::AnyBlock(block) => block.mutate_statement(index),
    }
}

fn resolve_statement_from_statement<'a>(
    any_statement: AnyStatementRef<'a>,
    index: usize,
) -> Option<AnyStatementRef<'a>> {
    match any_statement {
        AnyStatementRef::Statement(statement) => match statement {
            Statement::Do(do_statement) => do_statement.get_block().get_statement(index),
            Statement::Function(function) => function.get_block().get_statement(index),
            Statement::GenericFor(generic_for) => generic_for.get_block().get_statement(index),
            Statement::If(if_statement) => if_statement.get_statement(index),
            Statement::LocalFunction(function) => function.get_block().get_statement(index),
            Statement::NumericFor(numeric_for) => numeric_for.get_block().get_statement(index),
            Statement::Repeat(repeat) => repeat.get_block().get_statement(index),
            Statement::While(while_statement) => while_statement.get_block().get_statement(index),
            Statement::Assign(_)
            | Statement::Call(_)
            | Statement::CompoundAssign(_)
            | Statement::LocalAssign(_) => None,
        },
        AnyStatementRef::LastStatement(_) => None,
    }
}

fn resolve_statement_from_statement_mut<'a>(
    any_statement: AnyStatementRefMut<'a>,
    index: usize,
) -> Option<AnyStatementRefMut<'a>> {
    match any_statement {
        AnyStatementRefMut::Statement(statement) => match statement {
            Statement::Do(do_statement) => do_statement.mutate_block().mutate_statement(index),
            Statement::Function(function) => function.mutate_block().mutate_statement(index),
            Statement::GenericFor(generic_for) => {
                generic_for.mutate_block().mutate_statement(index)
            }
            Statement::If(if_statement) => if_statement.mutate_statement(index),
            Statement::LocalFunction(function) => function.mutate_block().mutate_statement(index),
            Statement::NumericFor(numeric_for) => {
                numeric_for.mutate_block().mutate_statement(index)
            }
            Statement::Repeat(repeat) => repeat.mutate_block().mutate_statement(index),
            Statement::While(while_statement) => {
                while_statement.mutate_block().mutate_statement(index)
            }
            Statement::Assign(_)
            | Statement::Call(_)
            | Statement::CompoundAssign(_)
            | Statement::LocalAssign(_) => None,
        },
        AnyStatementRefMut::LastStatement(_) => None,
    }
}

fn resolve_statement_from_expression<'a>(
    any_expression: AnyExpressionRef<'a>,
    index: usize,
) -> Option<AnyStatementRef<'a>> {
    match any_expression {
        AnyExpressionRef::Expression(expression) => match expression {
            Expression::Function(function) => function.get_block().get_statement(index),
            Expression::Binary(_)
            | Expression::Call(_)
            | Expression::False(_)
            | Expression::Field(_)
            | Expression::Identifier(_)
            | Expression::If(_)
            | Expression::Index(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::Parenthese(_)
            | Expression::String(_)
            | Expression::Table(_)
            | Expression::True(_)
            | Expression::Unary(_)
            | Expression::VariableArguments(_) => None,
        },
        AnyExpressionRef::Prefix(prefix) => match prefix {
            Prefix::Call(_)
            | Prefix::Field(_)
            | Prefix::Identifier(_)
            | Prefix::Index(_)
            | Prefix::Parenthese(_) => None,
        },
        AnyExpressionRef::Arguments(arguments) => match arguments {
            Arguments::Tuple(_) | Arguments::String(_) | Arguments::Table(_) => return None,
        },
        AnyExpressionRef::Variable(variable) => match variable {
            Variable::Identifier(_) | Variable::Field(_) | Variable::Index(_) => return None,
        },
    }
}

fn resolve_statement_from_expression_mut<'a>(
    any_expression: AnyExpressionRefMut<'a>,
    index: usize,
) -> Option<AnyStatementRefMut<'a>> {
    match any_expression {
        AnyExpressionRefMut::Expression(expression) => match expression {
            Expression::Function(function) => function.mutate_block().mutate_statement(index),
            Expression::Binary(_)
            | Expression::Call(_)
            | Expression::False(_)
            | Expression::Field(_)
            | Expression::Identifier(_)
            | Expression::If(_)
            | Expression::Index(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::Parenthese(_)
            | Expression::String(_)
            | Expression::Table(_)
            | Expression::True(_)
            | Expression::Unary(_)
            | Expression::VariableArguments(_) => None,
        },
        AnyExpressionRefMut::Prefix(prefix) => match prefix {
            Prefix::Call(_)
            | Prefix::Field(_)
            | Prefix::Identifier(_)
            | Prefix::Index(_)
            | Prefix::Parenthese(_) => None,
        },
        AnyExpressionRefMut::Arguments(arguments) => match arguments {
            Arguments::Tuple(_) | Arguments::String(_) | Arguments::Table(_) => return None,
        },
        AnyExpressionRefMut::Variable(variable) => match variable {
            Variable::Identifier(_) | Variable::Field(_) | Variable::Index(_) => return None,
        },
    }
}

fn resolve_expression<'a>(node: AnyNodeRef<'a>, index: usize) -> Option<AnyExpressionRef<'a>> {
    match node {
        AnyNodeRef::AnyStatement(any_statement) => {
            resolve_expression_from_statement(any_statement, index)
        }
        AnyNodeRef::AnyExpression(any_expression) => {
            resolve_expression_from_expression(any_expression, index)
        }
        AnyNodeRef::AnyBlock(_) => None,
    }
}

fn resolve_expression_mut<'a>(
    node: AnyNodeRefMut<'a>,
    index: usize,
) -> Option<AnyExpressionRefMut<'a>> {
    match node {
        AnyNodeRefMut::AnyStatement(any_statement) => {
            resolve_expression_from_statement_mut(any_statement, index)
        }
        AnyNodeRefMut::AnyExpression(any_expression) => {
            resolve_expression_from_expression_mut(any_expression, index)
        }
        AnyNodeRefMut::AnyBlock(_) => None,
    }
}

fn resolve_expression_from_statement<'a>(
    any_statement: AnyStatementRef<'a>,
    index: usize,
) -> Option<AnyExpressionRef<'a>> {
    let next_statement = match any_statement {
        AnyStatementRef::Statement(statement) => match statement {
            Statement::Assign(_) => todo!(),
            Statement::Do(_) => return None,
            Statement::Call(call) => resolve_call_expression(index, call)?.into(),
            Statement::CompoundAssign(assign) => match index {
                0 => assign.get_variable().into(),
                1 => assign.get_value().into(),
                _ => return None,
            },
            Statement::Function(_) => todo!(),
            Statement::GenericFor(_) => todo!(),
            Statement::If(_) => todo!(),
            Statement::LocalAssign(assign) => assign.iter_values().skip(index).next()?.into(),
            Statement::LocalFunction(_) => todo!(),
            Statement::NumericFor(_) => todo!(),
            Statement::Repeat(repeat) => match index {
                0 => repeat.get_condition().into(),
                _ => return None,
            },
            Statement::While(while_statement) => match index {
                0 => while_statement.get_condition().into(),
                _ => return None,
            },
        },
        AnyStatementRef::LastStatement(last_statement) => match last_statement {
            LastStatement::Break(_) => return None,
            LastStatement::Continue(_) => return None,
            LastStatement::Return(return_statement) => {
                AnyExpressionRef::from(return_statement.iter_expressions().skip(index).next()?)
            }
        },
    };
    Some(next_statement)
}

fn resolve_expression_from_statement_mut<'a>(
    any_statement: AnyStatementRefMut<'a>,
    index: usize,
) -> Option<AnyExpressionRefMut<'a>> {
    let next_statement = match any_statement {
        AnyStatementRefMut::Statement(statement) => match statement {
            Statement::Assign(_) => todo!(),
            Statement::Do(_) => todo!(),
            Statement::Call(call) => resolve_call_expression_mut(index, call)?.into(),
            Statement::CompoundAssign(assign) => match index {
                0 => assign.mutate_variable().into(),
                1 => assign.mutate_value().into(),
                _ => return None,
            },
            Statement::Function(_) => todo!(),
            Statement::GenericFor(_) => todo!(),
            Statement::If(_) => todo!(),
            Statement::LocalAssign(assign) => assign.iter_mut_values().skip(index).next()?.into(),
            Statement::LocalFunction(_) => todo!(),
            Statement::NumericFor(_) => todo!(),
            Statement::Repeat(repeat) => match index {
                0 => repeat.mutate_condition().into(),
                _ => return None,
            },
            Statement::While(while_statement) => match index {
                0 => while_statement.mutate_condition().into(),
                _ => return None,
            },
        },
        AnyStatementRefMut::LastStatement(last_statement) => match last_statement {
            LastStatement::Break(_) => return None,
            LastStatement::Continue(_) => return None,
            LastStatement::Return(return_statement) => AnyExpressionRefMut::from(
                return_statement.iter_mut_expressions().skip(index).next()?,
            ),
        },
    };
    Some(next_statement)
}

fn resolve_expression_from_expression<'a>(
    any_expression: AnyExpressionRef<'a>,
    index: usize,
) -> Option<AnyExpressionRef<'a>> {
    let next_expression = match any_expression {
        AnyExpressionRef::Expression(expression) => match expression {
            Expression::Binary(binary) => match index {
                0 => binary.left().into(),
                1 => binary.right().into(),
                _ => return None,
            },
            Expression::Call(call) => resolve_call_expression(index, call)?,
            Expression::Field(field) => resolve_field_expression(index, field)?,
            Expression::If(if_statement) => if_statement.get_expression(index)?.into(),
            Expression::Index(index_expression) => {
                resolve_index_expression(index, index_expression)?
            }
            Expression::Parenthese(parentheses) => {
                resolve_parentheses_expression(index, parentheses)?
            }
            Expression::Table(table) => resolve_table_expression(index, table)?,
            Expression::Unary(unary) => match index {
                0 => unary.get_expression().into(),
                _ => return None,
            },
            Expression::False(_)
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::True(_)
            | Expression::VariableArguments(_) => return None,
        },
        AnyExpressionRef::Prefix(prefix) => match prefix {
            Prefix::Call(call) => resolve_call_expression(index, call)?,
            Prefix::Field(field) => resolve_field_expression(index, field)?,
            Prefix::Index(index_expression) => resolve_index_expression(index, index_expression)?,
            Prefix::Parenthese(parentheses) => resolve_parentheses_expression(index, parentheses)?,
            Prefix::Identifier(_) => return None,
        },
        AnyExpressionRef::Arguments(arguments) => match arguments {
            Arguments::Tuple(tuple) => tuple.iter_values().skip(index).next()?.into(),
            Arguments::String(_) => return None,
            Arguments::Table(table) => resolve_table_expression(index, table)?,
        },
        AnyExpressionRef::Variable(variable) => match variable {
            Variable::Identifier(_) => return None,
            Variable::Field(field) => resolve_field_expression(index, field)?,
            Variable::Index(index_expression) => resolve_index_expression(index, index_expression)?,
        },
    };
    Some(next_expression)
}

fn resolve_expression_from_expression_mut<'a>(
    any_expression: AnyExpressionRefMut<'a>,
    index: usize,
) -> Option<AnyExpressionRefMut<'a>> {
    let next_expression = match any_expression {
        AnyExpressionRefMut::Expression(expression) => match expression {
            Expression::Binary(binary) => match index {
                0 => binary.mutate_left().into(),
                1 => binary.mutate_right().into(),
                _ => return None,
            },
            Expression::Call(call) => resolve_call_expression_mut(index, call)?,
            Expression::Field(field) => resolve_field_expression_mut(index, field)?,
            Expression::If(if_statement) => if_statement.mutate_expression(index)?.into(),
            Expression::Index(index_expression) => {
                resolve_index_expression_mut(index, index_expression)?
            }
            Expression::Parenthese(parentheses) => {
                resolve_parentheses_expression_mut(index, parentheses)?
            }
            Expression::Table(table) => resolve_table_expression_mut(index, table)?,
            Expression::Unary(unary) => match index {
                0 => unary.mutate_expression().into(),
                _ => return None,
            },
            Expression::False(_)
            | Expression::Function(_)
            | Expression::Identifier(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::True(_)
            | Expression::VariableArguments(_) => return None,
        },
        AnyExpressionRefMut::Prefix(prefix) => match prefix {
            Prefix::Call(call) => resolve_call_expression_mut(index, call)?,
            Prefix::Field(field) => resolve_field_expression_mut(index, field)?,
            Prefix::Index(index_expression) => {
                resolve_index_expression_mut(index, index_expression)?
            }
            Prefix::Parenthese(parentheses) => {
                resolve_parentheses_expression_mut(index, parentheses)?
            }
            Prefix::Identifier(_) => return None,
        },
        AnyExpressionRefMut::Arguments(arguments) => match arguments {
            Arguments::Tuple(tuple) => tuple.iter_mut_values().skip(index).next()?.into(),
            Arguments::String(_) => return None,
            Arguments::Table(table) => resolve_table_expression_mut(index, table)?,
        },
        AnyExpressionRefMut::Variable(variable) => match variable {
            Variable::Identifier(_) => return None,
            Variable::Field(field) => resolve_field_expression_mut(index, field)?,
            Variable::Index(index_expression) => {
                resolve_index_expression_mut(index, index_expression)?
            }
        },
    };
    Some(next_expression)
}

#[inline]
fn resolve_call_expression<'a>(
    index: usize,
    call: &'a FunctionCall,
) -> Option<AnyExpressionRef<'a>> {
    Some(match index {
        0 => call.get_prefix().into(),
        1 => call.get_arguments().into(),
        _ => return None,
    })
}

#[inline]
fn resolve_call_expression_mut<'a>(
    index: usize,
    call: &'a mut FunctionCall,
) -> Option<AnyExpressionRefMut<'a>> {
    Some(match index {
        0 => call.mutate_prefix().into(),
        1 => call.mutate_arguments().into(),
        _ => return None,
    })
}

#[inline]
fn resolve_field_expression<'a>(
    index: usize,
    field: &'a FieldExpression,
) -> Option<AnyExpressionRef<'a>> {
    match index {
        0 => Some(field.get_prefix().into()),
        _ => None,
    }
}

#[inline]
fn resolve_field_expression_mut<'a>(
    index: usize,
    field: &'a mut FieldExpression,
) -> Option<AnyExpressionRefMut<'a>> {
    match index {
        0 => Some(field.mutate_prefix().into()),
        _ => None,
    }
}

#[inline]
fn resolve_index_expression<'a>(
    index: usize,
    index_expression: &'a IndexExpression,
) -> Option<AnyExpressionRef<'a>> {
    Some(match index {
        0 => index_expression.get_prefix().into(),
        1 => index_expression.get_index().into(),
        _ => return None,
    })
}

#[inline]
fn resolve_index_expression_mut<'a>(
    index: usize,
    index_expression: &'a mut IndexExpression,
) -> Option<AnyExpressionRefMut<'a>> {
    Some(match index {
        0 => index_expression.mutate_prefix().into(),
        1 => index_expression.mutate_index().into(),
        _ => return None,
    })
}

#[inline]
fn resolve_parentheses_expression<'a>(
    index: usize,
    parentheses: &'a ParentheseExpression,
) -> Option<AnyExpressionRef<'a>> {
    match index {
        0 => Some(parentheses.inner_expression().into()),
        _ => None,
    }
}

#[inline]
fn resolve_parentheses_expression_mut<'a>(
    index: usize,
    parentheses: &'a mut ParentheseExpression,
) -> Option<AnyExpressionRefMut<'a>> {
    match index {
        0 => Some(parentheses.mutate_inner_expression().into()),
        _ => None,
    }
}

#[inline]
fn resolve_table_expression<'a>(
    _index: usize,
    _table: &'a TableExpression,
) -> Option<AnyExpressionRef<'a>> {
    todo!()
}

#[inline]
fn resolve_table_expression_mut<'a>(
    _index: usize,
    _table: &'a mut TableExpression,
) -> Option<AnyExpressionRefMut<'a>> {
    todo!()
}

const STATEMENT_DELIMITER: &str = "/";
const STATEMENT_DELIMITER_CHAR: char = '/';
const EXPRESSION_DELIMITER: &str = ":";
const EXPRESSION_DELIMITER_CHAR: char = ':';
const BLOCK_DELIMITER: &str = "#";
const BLOCK_DELIMITER_CHAR: char = '#';

impl FromStr for NodePathBuf {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut path = Self::default();

        if !string.is_ascii() {
            return Err(format_node_path_parse_error(
                string,
                "contains non-ascii characters",
            ));
        }

        for component in string.split_inclusive(|c| {
            matches!(
                c,
                STATEMENT_DELIMITER_CHAR | EXPRESSION_DELIMITER_CHAR | BLOCK_DELIMITER_CHAR
            )
        }) {
            if component.len() == 0 {
                return Err(format_node_path_parse_error(string, ""));
            }

            let (index_string, delimiter) = component.split_at(component.len().saturating_sub(1));

            let index = usize::from_str_radix(index_string, 10)
                .map_err(|err| format_node_path_parse_error(string, &err.to_string()))?;

            match delimiter {
                STATEMENT_DELIMITER => path.push_statement(index),
                EXPRESSION_DELIMITER => path.push_expression(index),
                BLOCK_DELIMITER => path.push_block(index),
                "" => {
                    return Err(format_node_path_parse_error(
                        string,
                        &format!("missing delimiter"),
                    ))
                }
                _ => {
                    return Err(format_node_path_parse_error(
                        string,
                        &format!("unexpected delimiter `{}`", delimiter),
                    ))
                }
            }
        }

        Ok(path)
    }
}

fn format_node_path_parse_error(input: &str, reason: &str) -> String {
    if reason.len() == 0 {
        format!("unable to parse path `{}`", input)
    } else {
        format!("unable to parse path `{}`: {}", input, reason)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_statement_paths {
        ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path = $path;

                    let parser = $crate::Parser::default();
                    let block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse($expect_code)
                        .expect("given expected code should parse");

                    assert_eq!(expected_block.total_len(), 1);
                    let expected_statement = expected_block.get_statement(0).unwrap();

                    pretty_assertions::assert_eq!(
                        path.resolve_statement(&block).expect("unable to resolve path"),
                        expected_statement
                    );
                }
            )*
        }
    }

    macro_rules! test_expression_paths {
        ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path = $path;

                    let parser = $crate::Parser::default();
                    let block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse(&format!("return {}", $expect_code))
                        .expect("given expected code should parse");

                    assert_eq!(expected_block.total_len(), 1);
                    let expected_expression = match expected_block
                        .get_last_statement()
                        .expect("expected block should have one return statement")
                    {
                        $crate::nodes::LastStatement::Return(statement) => {
                            assert_eq!(statement.len(), 1);
                            statement
                                .iter_expressions()
                                .next()
                                .expect("return statement should have one expression")
                        },
                        _ => panic!("return statement expected")
                    };

                    match path.resolve_expression(&block).expect("unable to resolve path") {
                        AnyExpressionRef::Expression(expression) => {
                            pretty_assertions::assert_eq!(expression, expected_expression);
                        }
                        AnyExpressionRef::Prefix(prefix) => {
                            pretty_assertions::assert_eq!(
                                &Expression::from(prefix.clone()),
                                expected_expression,
                            );
                        }
                        AnyExpressionRef::Arguments(_) => {
                            panic!("unable to compare arguments using this test macro")
                        }
                        AnyExpressionRef::Variable(_) => {
                            panic!("unable to compare variable using this test macro")
                        }
                    };
                }
            )*
        }
    }

    macro_rules! test_path_from_str {
        ($($name:ident ( $string:literal ) => $expected_path:expr ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path: NodePathBuf = $string
                        .parse()
                        .expect("unable to parse path");

                    pretty_assertions::assert_eq!(path, $expected_path);
                }
            )*
        }
    }

    macro_rules! test_path_strings {
        ($($name:ident => $path:literal ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path: NodePathBuf = $path
                        .parse()
                        .expect("unable to parse path");

                    let serialized = path.to_string();

                    pretty_assertions::assert_eq!($path, serialized);
                }
            )*
        }
    }

    fn new() -> NodePathBuf {
        NodePathBuf::default()
    }

    #[test]
    fn parent_of_root_is_none() {
        assert_eq!(new().parent(), None);
    }

    #[test]
    fn parent_of_statement_is_root() {
        assert_eq!(new().with_statement(0).parent(), Some(new().borrow()));
    }

    #[test]
    fn parent_of_statement_and_expression_is_statement() {
        assert_eq!(
            new().with_statement(0).with_expression(1).parent(),
            Some(new().with_statement(0).borrow())
        );
    }

    mod statement_paths {
        use super::*;

        test_statement_paths!(
            single_statement(
                new().with_statement(0),
                "do end"
            ) => "do end",
            second_statement(
                new().with_statement(1),
                "do end local var"
            ) => "local var",
            second_statement_is_last_statement(
                new().with_statement(1),
                "do end return 1"
            ) => "return 1",
            nested_local_definition(
                new().with_statement(0).with_statement(0),
                "do local a = 1 end"
            ) => "local a = 1",
            statement_with_nested_block(
                new().with_statement(0).with_statement(0),
                "do do while true do end end end"
            ) => "do while true do end end",
        );
    }

    mod expression_paths {
        use super::*;

        test_expression_paths!(
            return_statement_first_value(
                new().with_statement(0).with_expression(0),
                "return true"
            ) => "true",
            return_statement_second_value(
                new().with_statement(0).with_expression(1),
                "return true, nil"
            ) => "nil",
            return_statement_left_of_binary(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return condition or value"
            ) => "condition",
            return_statement_right_of_binary(
                new().with_statement(0).with_expression(0).with_expression(1),
                "return condition or value"
            ) => "value",
            return_statement_binary(
                new().with_statement(0).with_expression(0),
                "return condition or value"
            ) => "condition or value",
            return_statement_value_in_parens(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return (var)"
            ) => "var",
            return_statement_value_in_unary(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return not var"
            ) => "var",
            return_statement_if_expression_condition(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return if condition then result else other"
            ) => "condition",
            return_statement_if_expression_result(
                new().with_statement(0).with_expression(0).with_expression(1),
                "return if condition then result else other"
            ) => "result",
            return_statement_if_expression_else_result(
                new().with_statement(0).with_expression(0).with_expression(2),
                "return if condition then result else other"
            ) => "other",
            return_statement_elseif_expression_condition(
                new().with_statement(0).with_expression(0).with_expression(2),
                "return if condition then result elseif condition2 then result2 else other"
            ) => "condition2",
            return_statement_elseif_expression_result(
                new().with_statement(0).with_expression(0).with_expression(3),
                "return if condition then result elseif condition2 then result2 else other"
            ) => "result2",
            return_statement_elseif_expression_else_result(
                new().with_statement(0).with_expression(0).with_expression(4),
                "return if condition then result elseif condition2 then result2 else other"
            ) => "other",
            return_statement_index_prefix(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return value[key]"
            ) => "value",
            return_statement_index_value(
                new().with_statement(0).with_expression(0).with_expression(1),
                "return value[key]"
            ) => "key",
            return_statement_field_prefix(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return object.key"
            ) => "object",
            return_statement_call_prefix(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return object.callback()"
            ) => "object.callback",
            return_statement_call_first_argument(
                new().with_statement(0).with_expression(0).with_expression(1).with_expression(0),
                "return callback(true)"
            ) => "true",
            return_statement_call_second_argument(
                new().with_statement(0).with_expression(0).with_expression(1).with_expression(1),
                "return callback(1, 2)"
            ) => "2",
            local_assign_statement_first_value(
                new().with_statement(0).with_expression(0),
                "local condition = true"
            ) => "true",
            local_assign_statement_second_value(
                new().with_statement(0).with_expression(1),
                "local a, b = 1, 2"
            ) => "2",
            while_condition(
                new().with_statement(0).with_expression(0),
                "while condition do end"
            ) => "condition",
            repeat_condition(
                new().with_statement(0).with_expression(0),
                "repeat do end until condition == expected"
            ) => "condition == expected",
            statement_call_prefix(
                new().with_statement(0).with_expression(0),
                "print()"
            ) => "print",
            statement_call_first_argument(
                new().with_statement(0).with_expression(1).with_expression(0),
                "print('hello')"
            ) => "'hello'",
            statement_call_second_argument(
                new().with_statement(0).with_expression(1).with_expression(1),
                "print('variable', variable)"
            ) => "variable",
        );
    }

    mod path_from_string {
        use super::*;

        test_path_from_str!(
            statement_0("0/") => NodePathBuf::default().with_statement(0),
            statement_1("1/") => NodePathBuf::default().with_statement(1),
            statement_10("10/") => NodePathBuf::default().with_statement(10),
            statement_004("004/") => NodePathBuf::default().with_statement(4),
            statement_4_expr_1("4/1:") => NodePathBuf::default().with_statement(4).with_expression(1),
            statement_4_expr_1_statement_0("4/1:0/")
                => NodePathBuf::default().with_statement(4).with_expression(1).with_statement(0),
        );
    }

    mod path_strings {
        use super::*;

        test_path_strings!(
            statement_0 => "0/",
            statement_1 => "1/",
            statement_4  => "4/",
            statement_10  => "10/",
            statement_4_expr_1  => "4/1:",
            statement_4_expr_1_statement_0  => "4/1:0/",
        );
    }
}
