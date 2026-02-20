use crate::nodes::{
    Identifier, LiteralExpression, LiteralTable, StringExpression, Token, Trivia,
    TupleArgumentsTokens,
};

/// A list of function attributes.
///
/// Attributes can be either named (e.g., `@deprecated`) or grouped
/// (e.g., `@[attribute1, attribute2()]`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attributes {
    attributes: Vec<Attribute>,
}

impl Attributes {
    /// Creates a new empty list of attributes.
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    /// Adds an attribute to this list of attributes.
    pub fn with_attribute(mut self, attribute: impl Into<Attribute>) -> Self {
        self.attributes.push(attribute.into());
        self
    }

    /// Appends an attribute to this list of attributes.
    ///
    /// Empty attribute groups are silently ignored.
    pub fn append_attribute(&mut self, attribute: impl Into<Attribute>) {
        let attribute = attribute.into();
        if let Attribute::Group(list) = &attribute {
            if list.is_empty() {
                // don't append empty lists
                return;
            }
        }
        self.attributes.push(attribute);
    }

    /// Returns an iterator over the attributes in this list.
    pub fn iter_attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.iter()
    }

    /// Returns a mutable iterator over the attributes in this list.
    pub fn iter_mut_attributes(&mut self) -> impl Iterator<Item = &mut Attribute> {
        self.attributes.iter_mut()
    }

    /// Clears all attributes.
    pub fn clear_attributes(&mut self) {
        self.attributes.clear();
    }

    /// Returns whether this list of attributes is empty.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Returns the number of attributes in this list.
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Checks if an attribute with the given name exists in this list.
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|attr| match attr {
            Attribute::Name(named) => named.get_identifier().get_name() == name,
            Attribute::Group(group) => group.has_attribute(name),
        })
    }

    /// Filters attributes based on a predicate, keeping only those that return true.
    /// The predicate receives an immutable reference to each attribute.
    pub fn filter_attributes<F>(&mut self, predicate: F)
    where
        F: Fn(&Attribute) -> bool,
    {
        self.attributes.retain(predicate);
    }

    /// Filters attributes based on a mutable predicate, keeping only those that return true.
    /// The predicate receives a mutable reference to each attribute, allowing modifications.
    pub fn filter_mut_attributes<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut Attribute) -> bool,
    {
        self.attributes.retain_mut(predicate);
    }

    super::impl_token_fns!(iter = [attributes]);
}

/// Represents a function attribute.
///
/// Attributes can be either:
/// - Named: `@deprecated`
/// - Grouped: `@[attribute1, attribute2(args)]`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attribute {
    /// A named attribute (e.g., `@deprecated`)
    Name(NamedAttribute),
    /// A group of attributes (e.g., `@[attr1, attr2]`)
    Group(AttributeGroup),
}

impl Attribute {
    /// Clears all comments from the tokens in this node.
    pub fn clear_comments(&mut self) {
        match self {
            Self::Name(name) => name.clear_comments(),
            Self::Group(list) => list.clear_comments(),
        }
    }

    /// Clears all whitespaces information from the tokens in this node.
    pub fn clear_whitespaces(&mut self) {
        match self {
            Self::Name(name) => name.clear_whitespaces(),
            Self::Group(list) => list.clear_whitespaces(),
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            Self::Name(name) => name.replace_referenced_tokens(code),
            Self::Group(list) => list.replace_referenced_tokens(code),
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match self {
            Self::Name(name) => name.shift_token_line(amount),
            Self::Group(list) => list.shift_token_line(amount),
        }
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&Trivia) -> bool) {
        match self {
            Self::Name(name) => name.filter_comments(filter),
            Self::Group(list) => list.filter_comments(filter),
        }
    }
}

impl From<AttributeGroup> for Attribute {
    fn from(v: AttributeGroup) -> Self {
        Self::Group(v)
    }
}

impl From<AttributeGroupElement> for Attribute {
    fn from(v: AttributeGroupElement) -> Self {
        Self::Group(AttributeGroup::new(v))
    }
}

impl From<NamedAttribute> for Attribute {
    fn from(v: NamedAttribute) -> Self {
        Self::Name(v)
    }
}

/// A named function attribute.
///
/// Named attributes follow the syntax `@name`, such as `@deprecated`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedAttribute {
    name: Identifier,
    token: Option<Token>,
}

impl NamedAttribute {
    /// Creates a new named attribute with the given name.
    pub fn new(name: impl Into<Identifier>) -> Self {
        Self {
            name: name.into(),
            token: None,
        }
    }

    /// Attaches a token to this named attribute for the `@` symbol.
    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    /// Sets the token for this named attribute's `@` symbol.
    #[inline]
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    /// Returns the token for this named attribute's `@` symbol, if any.
    #[inline]
    pub fn get_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Returns the attribute's name.
    #[inline]
    pub fn get_identifier(&self) -> &Identifier {
        &self.name
    }

    /// Returns a mutable reference to the attribute's name.
    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut Identifier {
        &mut self.name
    }

    super::impl_token_fns!(target = [name] iter = [token]);
}

/// A group of attributes.
///
/// Attribute groups follow the syntax `@[attr1, attr2(args), ...]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeGroup {
    attributes: Vec<AttributeGroupElement>,
    tokens: Option<AttributeGroupTokens>,
}

impl AttributeGroup {
    /// Creates a new attribute group with a single attribute.
    pub fn new(attribute: impl Into<AttributeGroupElement>) -> Self {
        Self {
            attributes: vec![attribute.into()],
            tokens: None,
        }
    }

    /// Adds an attribute to this group.
    pub fn with_attribute(mut self, attribute: impl Into<AttributeGroupElement>) -> Self {
        self.attributes.push(attribute.into());
        self
    }

    /// Appends an attribute to this group.
    pub fn append_attribute(&mut self, attribute: impl Into<AttributeGroupElement>) {
        self.attributes.push(attribute.into());
    }

    /// Returns an iterator over the attributes in this group.
    pub fn iter_attributes(&self) -> impl Iterator<Item = &AttributeGroupElement> {
        self.attributes.iter()
    }

    /// Returns a mutable iterator over the attributes in this group.
    pub fn iter_mut_attributes(&mut self) -> impl Iterator<Item = &mut AttributeGroupElement> {
        self.attributes.iter_mut()
    }

    /// Attaches tokens to this attribute group.
    pub fn with_tokens(mut self, tokens: AttributeGroupTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Sets the tokens for this attribute group.
    pub fn set_tokens(&mut self, tokens: AttributeGroupTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns the tokens for this attribute group, if any.
    pub fn get_tokens(&self) -> Option<&AttributeGroupTokens> {
        self.tokens.as_ref()
    }

    /// Returns a mutable reference to the tokens for this attribute group, if any.
    pub fn mutate_tokens(&mut self) -> Option<&mut AttributeGroupTokens> {
        self.tokens.as_mut()
    }

    /// Returns whether this attribute group is empty.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Returns the number of attributes in this group.
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Checks if an attribute with the given name exists in this group.
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes
            .iter()
            .any(|elem| elem.name().get_name() == name)
    }

    /// Removes an attribute element at the specified index.
    /// Updates separator tokens if present.
    pub fn remove(&mut self, index: usize) {
        if index < self.attributes.len() {
            self.attributes.remove(index);

            // Update separators to match the new attributes length
            if let Some(tokens) = &mut self.tokens {
                if index < tokens.separators.len() {
                    tokens.separators.remove(index);
                }
            }
        }
    }

    /// Filters attribute elements based on a predicate, keeping only those that return true.
    /// Updates separator tokens to stay synchronized with the attributes.
    pub fn filter_attributes<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&AttributeGroupElement) -> bool,
    {
        let mut i = 0;
        while i < self.attributes.len() {
            if predicate(&self.attributes[i]) {
                i += 1;
            } else {
                self.remove(i);
            }
        }
    }

    /// Filters attribute elements with mutable access, keeping only those that return true.
    /// Updates separator tokens to stay synchronized with the attributes.
    pub fn filter_mut_attributes<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut AttributeGroupElement) -> bool,
    {
        let mut i = 0;
        while i < self.attributes.len() {
            if predicate(&mut self.attributes[i]) {
                i += 1;
            } else {
                self.remove(i);
            }
        }
    }

    super::impl_token_fns!(iter = [tokens, attributes]);
}

/// Tokens for an attribute group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeGroupTokens {
    /// Token for the `@[` opening.
    pub opening_attribute_list: Token,
    /// Token for the closing bracket `]`
    pub closing_bracket: Token,
    /// Tokens for the commas between entries.
    pub separators: Vec<Token>,
}

impl AttributeGroupTokens {
    super::impl_token_fns!(
        target = [opening_attribute_list, closing_bracket]
        iter = [separators]
    );
}

/// An element within an attribute group.
///
/// Elements can have optional arguments: `@[attr, attr_with_args(arg1, arg2)]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeGroupElement {
    name: Identifier,
    arguments: Option<AttributeArguments>,
}

impl AttributeGroupElement {
    /// Creates a new attribute group element with the given name.
    pub fn new(name: impl Into<Identifier>) -> Self {
        Self {
            name: name.into(),
            arguments: None,
        }
    }

    /// Attaches arguments to this attribute element.
    pub fn with_arguments(mut self, arguments: impl Into<AttributeArguments>) -> Self {
        self.arguments = Some(arguments.into());
        self
    }

    /// Returns the attribute element's name.
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns a mutable reference to the attribute element's name.
    pub fn mutate_name(&mut self) -> &mut Identifier {
        &mut self.name
    }

    /// Returns the attribute element's arguments, if any.
    pub fn get_arguments(&self) -> Option<&AttributeArguments> {
        self.arguments.as_ref()
    }

    /// Returns a mutable reference to the attribute element's arguments, if any.
    pub fn mutate_arguments(&mut self) -> Option<&mut AttributeArguments> {
        self.arguments.as_mut()
    }

    /// Sets the arguments for this attribute element.
    pub fn set_arguments(&mut self, arguments: impl Into<AttributeArguments>) {
        self.arguments = Some(arguments.into());
    }

    /// Removes the arguments from this attribute element.
    pub fn remove_arguments(&mut self) {
        self.arguments = None;
    }

    super::impl_token_fns!(target = [name] iter = [arguments]);
}

/// Arguments for an attribute element.
///
/// Attributes can accept tuple arguments, string literals, or table literals:
/// - Tuple: `@[attr(true, 0)]`
/// - String: `@[attr "text"]`
/// - Table: `@[attr { key = value }]`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeArguments {
    /// Tuple arguments: `attr("hello", 0)`
    Tuple(AttributeTupleArguments),
    /// String argument: `attr "text"`
    String(StringExpression),
    /// Table argument: `attr { key = value }`
    Table(LiteralTable),
}

impl AttributeArguments {
    /// Clears all comments from the tokens in this node.
    pub fn clear_comments(&mut self) {
        match self {
            Self::Tuple(tuple) => tuple.clear_comments(),
            Self::String(string) => string.clear_comments(),
            Self::Table(table) => table.clear_comments(),
        }
    }

    /// Clears all whitespaces information from the tokens in this node.
    pub fn clear_whitespaces(&mut self) {
        match self {
            Self::Tuple(tuple) => tuple.clear_whitespaces(),
            Self::String(string) => string.clear_whitespaces(),
            Self::Table(table) => table.clear_whitespaces(),
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            Self::Tuple(tuple) => tuple.replace_referenced_tokens(code),
            Self::String(string) => string.replace_referenced_tokens(code),
            Self::Table(table) => table.replace_referenced_tokens(code),
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match self {
            Self::Tuple(tuple) => tuple.shift_token_line(amount),
            Self::String(string) => string.shift_token_line(amount),
            Self::Table(table) => table.shift_token_line(amount),
        }
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&Trivia) -> bool) {
        match self {
            Self::Tuple(tuple) => tuple.filter_comments(filter),
            Self::String(string) => string.filter_comments(filter),
            Self::Table(table) => table.filter_comments(filter),
        }
    }
}

impl From<LiteralTable> for AttributeArguments {
    fn from(v: LiteralTable) -> Self {
        Self::Table(v)
    }
}

impl From<StringExpression> for AttributeArguments {
    fn from(v: StringExpression) -> Self {
        Self::String(v)
    }
}

impl From<AttributeTupleArguments> for AttributeArguments {
    fn from(v: AttributeTupleArguments) -> Self {
        Self::Tuple(v)
    }
}

/// Tuple arguments for an attribute element.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttributeTupleArguments {
    values: Vec<LiteralExpression>,
    tokens: Option<TupleArgumentsTokens>,
}

impl AttributeTupleArguments {
    pub fn with_value(mut self, value: impl Into<LiteralExpression>) -> Self {
        self.push(value.into());
        self
    }

    pub fn push(&mut self, argument: impl Into<LiteralExpression>) {
        let argument = argument.into();
        let initial_len = self.values.len();

        self.values.push(argument);

        if initial_len != 0 {
            if let Some(tokens) = &mut self.tokens {
                if tokens.commas.len() == initial_len - 1 {
                    tokens.commas.push(Token::from_content(","));
                }
            }
        }
    }

    /// Inserts an argument at the specified index.
    pub fn insert(&mut self, index: usize, argument: impl Into<LiteralExpression>) {
        if index >= self.values.len() {
            self.push(argument.into());
        } else {
            self.values.insert(index, argument.into());

            if let Some(tokens) = &mut self.tokens {
                if index <= tokens.commas.len() {
                    tokens.commas.insert(index, Token::from_content(","));
                }
            }
        }
    }

    /// Returns the number of arguments in this tuple.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether this tuple has no arguments.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns an iterator over the argument values.
    pub fn iter_values(&self) -> impl Iterator<Item = &LiteralExpression> {
        self.values.iter()
    }

    /// Returns a mutable iterator over the argument values.
    pub fn iter_mut_values(&mut self) -> impl Iterator<Item = &mut LiteralExpression> {
        self.values.iter_mut()
    }

    /// Returns the tokens for this tuple arguments, if any.
    pub fn get_tokens(&self) -> Option<&TupleArgumentsTokens> {
        self.tokens.as_ref()
    }

    /// Sets the tokens for this tuple arguments.
    pub fn set_tokens(&mut self, tokens: TupleArgumentsTokens) {
        self.tokens = Some(tokens);
    }

    /// Attaches tokens to this tuple arguments.
    pub fn with_tokens(mut self, tokens: TupleArgumentsTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    super::impl_token_fns!(iter = [tokens]);
}
