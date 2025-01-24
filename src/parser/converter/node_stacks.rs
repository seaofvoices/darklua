use crate::{
    nodes::{
        Arguments, Block, Expression, FunctionReturnType, GenericTypePack, LastStatement, Prefix,
        Statement, TableEntry, Type, TypePack, TypeParameters, TypedIdentifier, Variable,
        VariadicTypePack,
    },
    parser::ast_converter::ConvertError,
};

use super::{InterpolationSegment, StringSegment, ValueSegment};

#[derive(Debug, Default)]
pub(crate) struct NodeStacks {
    blocks: Vec<Block>,
    typed_identifiers: Vec<TypedIdentifier>,
    statements: Vec<Statement>,
    last_statements: Vec<LastStatement>,
    expressions: Vec<Expression>,
    table_entries: Vec<TableEntry>,
    prefixes: Vec<Prefix>,
    arguments: Vec<Arguments>,
    variables: Vec<Variable>,
    types: Vec<Type>,
    function_return_types: Vec<FunctionReturnType>,
    variadic_type_packs: Vec<VariadicTypePack>,
    generic_type_packs: Vec<GenericTypePack>,
    type_parameters: Vec<TypeParameters>,
    type_packs: Vec<TypePack>,
    interpolation_segments: Vec<InterpolationSegment>,
}

impl NodeStacks {
    pub(crate) fn pop_block(&mut self) -> Result<Block, ConvertError> {
        self.blocks
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Block" })
    }

    pub(crate) fn pop_typed_identifier(&mut self) -> Result<TypedIdentifier, ConvertError> {
        self.typed_identifiers
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "TypedIdentifier",
            })
    }

    pub(crate) fn pop_typed_identifiers(
        &mut self,
        n: usize,
    ) -> Result<Vec<TypedIdentifier>, ConvertError> {
        std::iter::repeat_with(|| self.pop_typed_identifier())
            .take(n)
            .collect()
    }

    pub(crate) fn pop_statement(&mut self) -> Result<Statement, ConvertError> {
        self.statements
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Statement" })
    }

    pub(crate) fn pop_statements(&mut self, n: usize) -> Result<Vec<Statement>, ConvertError> {
        std::iter::repeat_with(|| self.pop_statement())
            .take(n)
            .collect()
    }

    pub(crate) fn pop_last_statement(&mut self) -> Result<LastStatement, ConvertError> {
        self.last_statements
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "LastStatement",
            })
    }

    pub(crate) fn pop_expression(&mut self) -> Result<Expression, ConvertError> {
        self.expressions
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Expression" })
    }

    pub(crate) fn pop_expressions(&mut self, n: usize) -> Result<Vec<Expression>, ConvertError> {
        std::iter::repeat_with(|| self.pop_expression())
            .take(n)
            .collect()
    }

    pub(crate) fn pop_table_entry(&mut self) -> Result<TableEntry, ConvertError> {
        self.table_entries
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "TableEntry" })
    }

    pub(crate) fn pop_table_entries(&mut self, n: usize) -> Result<Vec<TableEntry>, ConvertError> {
        std::iter::repeat_with(|| self.pop_table_entry())
            .take(n)
            .collect()
    }

    pub(crate) fn pop_prefix(&mut self) -> Result<Prefix, ConvertError> {
        self.prefixes
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Prefix" })
    }

    pub(crate) fn pop_variable(&mut self) -> Result<Variable, ConvertError> {
        self.variables
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Variable" })
    }

    pub(crate) fn pop_variables(&mut self, n: usize) -> Result<Vec<Variable>, ConvertError> {
        std::iter::repeat_with(|| self.pop_variable())
            .take(n)
            .collect()
    }

    pub(crate) fn pop_arguments(&mut self) -> Result<Arguments, ConvertError> {
        self.arguments
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Arguments" })
    }

    pub(crate) fn pop_type(&mut self) -> Result<Type, ConvertError> {
        self.types
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Type" })
    }

    pub(crate) fn pop_variadic_type_pack(&mut self) -> Result<VariadicTypePack, ConvertError> {
        self.variadic_type_packs
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "VariadicTypePack",
            })
    }

    pub(crate) fn pop_generic_type_pack(&mut self) -> Result<GenericTypePack, ConvertError> {
        self.generic_type_packs
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "GenericTypePack",
            })
    }

    pub(crate) fn pop_function_return_type(&mut self) -> Result<FunctionReturnType, ConvertError> {
        self.function_return_types
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "FunctionReturnType",
            })
    }

    pub(crate) fn pop_type_parameters(&mut self) -> Result<TypeParameters, ConvertError> {
        self.type_parameters
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "TypeParameters",
            })
    }

    pub(crate) fn pop_type_pack(&mut self) -> Result<TypePack, ConvertError> {
        self.type_packs
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "TypePack" })
    }

    pub(crate) fn pop_interpolation_segment(&mut self) -> Result<InterpolationSegment, ConvertError> {
        self.interpolation_segments
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "InterpolationSegment" })
    }

    pub(crate) fn pop_interpolation_segments(&mut self, n: usize) -> Result<Vec<InterpolationSegment>, ConvertError> {
        std::iter::repeat_with(|| self.pop_interpolation_segment())
            .take(n)
            .collect()
    }
}

pub(crate) trait PushNode<T> {
    fn push(&mut self, node: T);
}

impl PushNode<Block> for NodeStacks {
    fn push(&mut self, node: Block) {
        self.blocks.push(node);
    }
}

impl PushNode<TypedIdentifier> for NodeStacks {
    fn push(&mut self, node: TypedIdentifier) {
        self.typed_identifiers.push(node);
    }
}

impl PushNode<Statement> for NodeStacks {
    fn push(&mut self, node: Statement) {
        self.statements.push(node);
    }
}

impl PushNode<LastStatement> for NodeStacks {
    fn push(&mut self, node: LastStatement) {
        self.last_statements.push(node);
    }
}

impl PushNode<Expression> for NodeStacks {
    fn push(&mut self, node: Expression) {
        self.expressions.push(node);
    }
}

impl PushNode<TableEntry> for NodeStacks {
    fn push(&mut self, node: TableEntry) {
        self.table_entries.push(node);
    }
}

impl PushNode<Prefix> for NodeStacks {
    fn push(&mut self, node: Prefix) {
        self.prefixes.push(node);
    }
}

impl PushNode<Arguments> for NodeStacks {
    fn push(&mut self, node: Arguments) {
        self.arguments.push(node);
    }
}

impl PushNode<Variable> for NodeStacks {
    fn push(&mut self, node: Variable) {
        self.variables.push(node);
    }
}

impl PushNode<Type> for NodeStacks {
    fn push(&mut self, node: Type) {
        self.types.push(node);
    }
}

impl PushNode<FunctionReturnType> for NodeStacks {
    fn push(&mut self, node: FunctionReturnType) {
        self.function_return_types.push(node);
    }
}

impl PushNode<VariadicTypePack> for NodeStacks {
    fn push(&mut self, node: VariadicTypePack) {
        self.variadic_type_packs.push(node);
    }
}

impl PushNode<GenericTypePack> for NodeStacks {
    fn push(&mut self, node: GenericTypePack) {
        self.generic_type_packs.push(node);
    }
}

impl PushNode<TypeParameters> for NodeStacks {
    fn push(&mut self, node: TypeParameters) {
        self.type_parameters.push(node);
    }
}

impl PushNode<TypePack> for NodeStacks {
    fn push(&mut self, node: TypePack) {
        self.type_packs.push(node);
    }
}

impl PushNode<InterpolationSegment> for NodeStacks {
    fn push(&mut self, node: InterpolationSegment) {
        self.interpolation_segments.push(node);
    }
}

impl PushNode<StringSegment> for NodeStacks {
    fn push(&mut self, node: StringSegment) {
        self.interpolation_segments
            .push(InterpolationSegment::String(node));
    }
}

impl PushNode<ValueSegment> for NodeStacks {
    fn push(&mut self, node: ValueSegment) {
        self.interpolation_segments
            .push(InterpolationSegment::Value(node));
    }
}
