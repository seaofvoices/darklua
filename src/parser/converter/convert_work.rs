use crate::{nodes::*, parser::ast_converter::ConvertError};

pub(crate) struct WorkStack<C: std::fmt::Debug> {
    work_stack: Vec<ConvertWork<C>>,
    temporary_stack: Vec<ConvertWork<C>>,
}

impl<C: std::fmt::Debug> WorkStack<C> {
    pub(crate) fn new_with(initial_work: ConvertWork<C>) -> Self {
        Self {
            work_stack: vec![initial_work],
            temporary_stack: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<ConvertWork<C>> {
        if !self.temporary_stack.is_empty() {
            self.work_stack.extend(self.temporary_stack.drain(..));
        }
        self.work_stack.pop()
    }
}

impl<C: std::fmt::Debug> WorkScheduler for WorkStack<C> {
    type Convert = C;

    fn new() -> Self {
        Self {
            work_stack: Vec::new(),
            temporary_stack: Vec::new(),
        }
    }

    fn push2(&mut self, new: ConvertWork<Self::Convert>) {
        println!("PUSH WORK: {:#?}", new);
        if !self.temporary_stack.is_empty() {
            self.work_stack.extend(self.temporary_stack.drain(..));
        }
        self.work_stack.push(new);
    }

    fn defer(&mut self, new: ConvertWork<Self::Convert>) {
        self.temporary_stack.push(new);
    }

    fn defer_merge_reverse(&mut self, work: Self) {
        self.temporary_stack
            .extend(work.work_stack.into_iter().rev());
        self.temporary_stack
            .extend(work.temporary_stack.into_iter().rev());
    }

    fn flush(&mut self) {
        self.work_stack.extend(self.temporary_stack.drain(..));
    }

    fn push_and_flush(&mut self, new: ConvertWork<Self::Convert>) {
        self.work_stack.push(new);
        self.work_stack.extend(self.temporary_stack.drain(..));
        println!("PUSH AND FLUSH {:#?}", self.work_stack);
    }
}

pub(crate) trait WorkScheduler {
    type Convert: std::fmt::Debug;

    fn new() -> Self;

    fn push2(&mut self, new: ConvertWork<Self::Convert>);

    fn defer(&mut self, new: ConvertWork<Self::Convert>);

    fn defer_merge_reverse(&mut self, work: Self);

    fn flush(&mut self);

    fn push_and_flush(&mut self, new: ConvertWork<Self::Convert>);
}

pub(crate) trait Convertable {
    type Convert;

    fn convert(
        self,
        work: &mut impl WorkScheduler<Convert = Self::Convert>,
    ) -> Result<(), ConvertError>;
}

#[derive(Debug)]
pub(crate) enum ConvertWork<C: std::fmt::Debug> {
    Convert(C),
    PushStatement(Statement),
    PushLastStatement(LastStatement),
    PushExpression(Expression),
    PushPrefix(Prefix),
    PushArguments(Arguments),
    PushTypedIdentifier(TypedIdentifier),
    PushVariable(Variable),
    PushType(Type),
    MakeBlock {
        statement_count: usize,
        has_last_statement: bool,
        tokens: Option<BlockTokens>,
    },
    MakeDoStatement {
        tokens: Option<DoTokens>,
    },
    MakeRepeatStatement {
        tokens: Option<RepeatTokens>,
    },
    MakeWhileStatement {
        tokens: Option<WhileTokens>,
    },
    MakeNumericForStatement {
        has_step_expression: bool,
        tokens: Option<NumericForTokens>,
    },
    MakeGenericForStatement {
        identifier_count: usize,
        expression_count: usize,
        tokens: Option<GenericForTokens>,
    },
    MakeLocalFunctionStatement {
        identifier: Identifier,
        parameter_count: usize,
        is_variadic: bool,
        tokens: Option<LocalFunctionTokens>,
    },
    MakeFunctionStatement {
        function_name: FunctionName,
        parameter_count: usize,
        is_variadic: bool,
        tokens: Option<FunctionBodyTokens>,
    },
    MakeLocalAssignStatement {
        identifier_count: usize,
        expression_count: usize,
        tokens: Option<LocalAssignTokens>,
    },
    MakeAssignStatement {
        variable_count: usize,
        expression_count: usize,
        tokens: Option<AssignTokens>,
    },
    MakeIfStatement {
        elseif_tokens: Vec<Option<IfBranchTokens>>,
        has_else_block: bool,
        tokens: Option<IfStatementTokens>,
    },
    MakeCompoundAssignStatement {
        operator: CompoundOperator,
        tokens: Option<CompoundAssignTokens>,
    },
    MakeReturn {
        expression_count: usize,
        tokens: Option<ReturnTokens>,
    },
    MakeBinaryExpression {
        operator: BinaryOperator,
        token: Option<Token>,
    },
    MakeUnaryExpression {
        operator: UnaryOperator,
        token: Option<Token>,
    },
    MakeParentheseExpression {
        tokens: Option<ParentheseTokens>,
    },
    MakeIfExpression {
        elseif_branch_count: usize, // tokens:
    },
    MakeFunctionExpression {
        parameter_count: usize,
        is_variadic: bool,
        tokens: Option<FunctionBodyTokens>,
    },
    MakeFunctionCallExpression {
        // tokens:
    },
    MakeFunctionCallStatement,
    MakeTypeDeclarationStatement {
        // tokens:
    },
    MakePrefixFromExpression,
    MakeIndexPrefix {
        tokens: Option<IndexExpressionTokens>,
    },
    MakeFieldPrefix {
        identifier: Identifier,
        token: Option<Token>,
    },
    MakeCallPrefix {
        method: Option<Identifier>,
        tokens: Option<FunctionCallTokens>,
    },
    MakeArgumentsFromExpressions {
        expression_count: usize,
        tokens: Option<TupleArgumentsTokens>,
    },
    MakeArgumentsFromTableEntries {
        entry_count: usize,
        tokens: Option<TableTokens>,
    },
    MakeTableExpression {
        entry_count: usize,
        tokens: Option<TableTokens>,
    },
    MakeFieldTableEntry {
        identifier: Identifier,
        token: Option<Token>,
    },
    MakeIndexTableEntry {
        tokens: Option<TableIndexEntryTokens>,
    },
    MakeValueTableEntry,
    MakeVariable,
    MakePrefixExpression,
    MakeTypedIdentifier {
        identifier: Identifier,
        token: Option<Token>,
    },
    MakeInterpolatedString {
        // interpolated_string: &'a ast::types::InterpolatedString,
    },
    MakeFunctionReturnType {
        // type_info: &'a ast::types::TypeInfo,
    },
    MakeVariadicTypePack {
        // ellipse: &'a tokenizer::TokenReference,
    },
    MakeArrayType {
        // braces: &'a ast::span::ContainedSpan,
    },
    MakeOptionalType {
        // question_mark: &'a tokenizer::TokenReference,
    },
    MakeUnionType {
        // operator: &'a tokenizer::TokenReference,
    },
    MakeIntersectionType {
        // operator: &'a tokenizer::TokenReference,
    },
    MakeTableType {
        // braces: &'a ast::span::ContainedSpan,
        // fields: &'a ast::punctuated::Punctuated<ast::types::TypeField>,
    },
    MakeExpressionType {
        // typeof_token: &'a tokenizer::TokenReference,
        // parentheses: &'a ast::span::ContainedSpan,
    },
    MakeFunctionType {
        // generics: &'a Option<ast::types::GenericDeclaration>,
        // parentheses: &'a ast::span::ContainedSpan,
        // arguments: &'a ast::punctuated::Punctuated<ast::types::TypeArgument>,
        // arrow: &'a tokenizer::TokenReference,
    },
    MakeGenericType {
        // base: &'a tokenizer::TokenReference,
        // module: Option<(&'a tokenizer::TokenReference, &'a tokenizer::TokenReference)>,
    },
    MakeTypeParameters {
        // arrows: &'a ast::span::ContainedSpan,
        // generics: &'a ast::punctuated::Punctuated<ast::types::TypeInfo>,
    },
    MakeTypeCast {
        // type_assertion: &'a ast::types::TypeAssertion,
    },
    MakeParentheseType {
        // parentheses: &'a ast::span::ContainedSpan,
    },
    MakeTypePack {
        // parentheses: &'a ast::span::ContainedSpan,
        // types: &'a ast::punctuated::Punctuated<ast::types::TypeInfo>,
    },
}

impl<C: std::fmt::Debug> From<C> for ConvertWork<C> {
    fn from(value: C) -> Self {
        Self::Convert(value)
    }
}
