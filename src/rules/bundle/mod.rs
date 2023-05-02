mod expression_serializer;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::mem;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::frontend::DarkluaResult;
use crate::nodes::{
    Arguments, AssignStatement, Block, DoStatement, Expression, FieldExpression, FunctionCall,
    Identifier, LastStatement, LocalAssignStatement, Prefix, Statement, TableExpression,
};
use crate::process::utils::{generate_identifier, identifier_permutator, CharPermutator};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    verify_required_properties, Context, ContextBuilder, ReplaceReferencedTokens, Rule,
    RuleConfiguration, RuleConfigurationError, RuleProcessResult, RuleProperties,
    RulePropertyValue,
};
use crate::utils::normalize_path;
use crate::{utils, DarkluaError, Parser, Resources};

pub(crate) use expression_serializer::{to_expression, LuaSerializerError};

#[derive(Debug)]
struct RequirePathProcessor<'a, 'b> {
    source: PathBuf,
    extra_module_relative_location: Option<&'a Path>,
    modules_identifier: &'a str,
    path_require_mode: &'a PathRequireMode,
    module_name_permutator: CharPermutator,
    module_cache: HashMap<PathBuf, Expression>,
    skip_module_paths: HashSet<PathBuf>,
    module_definitions: Vec<Block>,
    parser: &'a Parser,
    resources: &'b Resources,
    errors: Vec<String>,
    defined_modules: usize,
}

impl<'a, 'b> RequirePathProcessor<'a, 'b> {
    fn new(
        source: impl Into<PathBuf>,
        extra_module_relative_location: Option<&'a Path>,
        modules_identifier: &'a str,
        path_require_mode: &'a PathRequireMode,
        resources: &'b Resources,
        parser: &'a Parser,
    ) -> Self {
        Self {
            source: source.into(),
            extra_module_relative_location,
            modules_identifier,
            path_require_mode,
            module_name_permutator: identifier_permutator(),
            module_cache: Default::default(),
            skip_module_paths: Default::default(),
            module_definitions: Vec::new(),
            resources,
            parser,
            errors: Vec::new(),
            defined_modules: 0,
        }
    }

    fn result(self) -> RuleProcessResult {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("\n- "))
        }
    }

    fn require_call(&self, call: &FunctionCall) -> Option<PathBuf> {
        if call.get_method().is_some() {
            return None;
        }

        match call.get_prefix() {
            Prefix::Identifier(identifier) if identifier.get_name() == "require" => {
                match call.get_arguments() {
                    Arguments::String(string) => Some(string.get_value()),
                    Arguments::Tuple(tuple) if tuple.len() == 1 => {
                        let expression = tuple.iter_values().next().unwrap();

                        match expression {
                            Expression::String(string) => Some(string.get_value()),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
        .map(Path::new)
        .map(utils::normalize_path)
    }

    fn inline_call(&mut self, call: &FunctionCall) -> Option<Expression> {
        let require_path = self.require_call(call)?;

        log::trace!("found require call to path `{}`", require_path.display());

        if self.skip_module_paths.contains(&require_path) {
            return None;
        }

        match self.try_inline_call(&require_path) {
            Ok(expression) => Some(expression),
            Err(error) => {
                self.errors.push(error.to_string());
                self.skip_module_paths.insert(require_path);
                None
            }
        }
    }

    fn try_inline_call(&mut self, require_path: &Path) -> DarkluaResult<Expression> {
        let require_path = self.normalize_require_path(require_path)?;
        if let Some(expression) = self.module_cache.get(&require_path) {
            Ok(expression.clone())
        } else {
            let (module_name, block) = match self.require_resource(&require_path)? {
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
                        // block.set_last_statement(ReturnStatement::default().with_expression(
                        //     FieldExpression::new(
                        //         Identifier::from(self.modules_identifier),
                        //         module_name.clone(),
                        //     ),
                        // ));

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
                    let block = Block::default()
                        .with_statement(AssignStatement::from_variable(
                            FieldExpression::new(
                                Identifier::from(self.modules_identifier),
                                module_name.clone(),
                            ),
                            expression,
                        ))
                        // .with_last_statement(
                        //     ReturnStatement::default()
                        //         .with_expression(FieldExpression::new(
                        //             Identifier::from(self.modules_identifier),
                        //             module_name.clone(),
                        //         ))
                        //         .into(),
                        // )
                        ;

                    (module_name, block)
                }
            };
            // let module_function = FunctionExpression::from_block(block);

            self.module_definitions.push(block);

            let module_value: Expression =
                FieldExpression::new(Identifier::from(self.modules_identifier), module_name).into();
            self.defined_modules += 1;
            self.module_cache
                .insert(require_path.to_path_buf(), module_value.clone());

            // Ok(FunctionCall::from_prefix(ParentheseExpression::new(module_function)).into())
            Ok(module_value)
        }
    }

    fn require_resource(&mut self, path: impl AsRef<Path>) -> DarkluaResult<RequiredResource> {
        let path = path.as_ref();
        log::trace!("look for resource `{}`", path.display());
        let content = self.resources.get(path).map_err(DarkluaError::from)?;

        match path.extension() {
            Some(extension) => match extension.to_string_lossy().as_ref() {
                "lua" | "luau" => {
                    let mut block = self.parser.parse(&content).map_err(|parser_error| {
                        DarkluaError::parser_error(path.to_path_buf(), parser_error)
                    })?;

                    let path_buf = path.to_path_buf();
                    let current_source = mem::replace(&mut self.source, path.to_path_buf());
                    DefaultVisitor::visit_block(&mut block, self);
                    self.source = current_source;

                    if self.parser.is_preserving_tokens() {
                        let context =
                            ContextBuilder::new(path_buf.clone(), self.resources, &content).build();
                        // run `replace_referenced_tokens` rule to avoid generating invalid code
                        // when using the token-based generator
                        let replace_tokens = ReplaceReferencedTokens::default();
                        replace_tokens
                            .process(&mut block, &context)
                            .map_err(|rule_error| {
                                let error = DarkluaError::orphan_rule_error(
                                    path_buf.clone(),
                                    &replace_tokens,
                                    rule_error,
                                );
                                log::trace!(
                                    "[{}] rule `{}` errored: {}",
                                    path_buf.display(),
                                    replace_tokens.get_name(),
                                    error
                                );
                                error
                            })?;
                    }
                    Ok(RequiredResource::Block(block))
                }
                "json" | "json5" => transcode(json5::from_str::<serde_json::Value>(&content)),
                "yml" | "yaml" => transcode(serde_yaml::from_str::<serde_yaml::Value>(&content)),
                "toml" => transcode(toml::from_str::<toml::Value>(&content)),
                _ => Err(DarkluaError::invalid_resource_extension(path)),
            },
            None => unreachable!("extension should be defined"),
        }
    }

    fn normalize_require_path(&self, path: impl Into<PathBuf>) -> Result<PathBuf, DarkluaError> {
        let mut path: PathBuf = path.into();
        if path.has_root() {
            let extra_module_relative_location =
                self.extra_module_relative_location.ok_or_else(|| {
                    DarkluaError::invalid_resource_path(
                        path.display().to_string(),
                        "unable to obtain configuration file location",
                    )
                })?;

            let mut components = path.components().skip(1);
            let root = components.next().ok_or_else(|| {
                DarkluaError::invalid_resource_path(
                    path.display().to_string(),
                    "unable to obtain source from root path",
                )
            })?;
            let source_name = root.as_os_str().to_str().ok_or_else(|| {
                DarkluaError::invalid_resource_path(
                    path.display().to_string(),
                    "unable to read source name",
                )
            })?;

            let source_components = self
                .path_require_mode
                .get_source(source_name)
                .ok_or_else(|| {
                    DarkluaError::invalid_resource_path(
                        path.display().to_string(),
                        format!("unknown source name `{}`", source_name),
                    )
                })?
                .components()
                .chain(components);

            path = extra_module_relative_location.join(PathBuf::from_iter(source_components));
        } else if path.is_relative() {
            if self.resources.is_file(&self.source)? {
                let mut new_path = self.source.clone();
                new_path.pop();
                new_path.push(path);
                path = new_path;
            } else {
                path = self.source.join(path);
            }
        }
        if self
            .resources
            .is_directory(&path)
            .map_err(DarkluaError::from)?
        {
            path.push(self.path_require_mode.module_folder_name());
        }
        if path.extension().is_none() {
            path.set_extension("lua");
            if !self.resources.exists(&path)? {
                path.set_extension("luau");
            }
        };
        Ok(normalize_path(path))
    }
}

impl<'a, 'b> NodeProcessor for RequirePathProcessor<'a, 'b> {
    fn process_expression(&mut self, expression: &mut Expression) {
        if let Expression::Call(call) = expression {
            if let Some(replace_with) = self.inline_call(call) {
                *expression = replace_with;
            }
        }
    }

    fn process_prefix_expression(&mut self, prefix: &mut Prefix) {
        if let Prefix::Call(call) = prefix {
            if let Some(replace_with) = self.inline_call(call) {
                *prefix = replace_with.into();
            }
        }
    }

    fn process_statement(&mut self, statement: &mut Statement) {
        if let Statement::Call(call) = statement {
            if let Some(replace_with) = self.inline_call(call) {
                if let Expression::Call(replace_with) = replace_with {
                    *call = *replace_with;
                } else {
                    *statement = convert_expression_to_statement(replace_with);
                }
            }
        }
    }
}

fn convert_expression_to_statement(expression: Expression) -> Statement {
    DoStatement::new(
        Block::default()
            .with_statement(LocalAssignStatement::from_variable("_").with_value(expression)),
    )
    .into()
}

pub const BUNDLER_RULE_NAME: &str = "bundler";

/// A rule that inlines required modules
#[derive(Debug)]
pub(crate) struct Bundler {
    parser: Parser,
    extra_module_relative_location: Option<PathBuf>,
    modules_identifier: String,
    require_mode: RequireMode,
}

impl Bundler {
    pub(crate) fn with_modules_identifier(mut self, modules_identifier: impl Into<String>) -> Self {
        self.modules_identifier = modules_identifier.into();
        self
    }

    pub(crate) fn with_require_mode(mut self, require_mode: impl Into<RequireMode>) -> Self {
        self.require_mode = require_mode.into();
        self
    }

    pub(crate) fn with_parser(mut self, parser: Parser) -> Self {
        self.parser = parser;
        self
    }

    pub(crate) fn with_configuration_location(mut self, location: impl AsRef<Path>) -> Self {
        let location = location
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        self.extra_module_relative_location = Some(location);
        self
    }
}

impl Rule for Bundler {
    fn process(&self, block: &mut Block, context: &Context) -> RuleProcessResult {
        self.require_mode.process_block(
            block,
            context.current_path().to_path_buf(),
            self.extra_module_relative_location.as_deref(),
            &self.modules_identifier,
            context.resources(),
            &self.parser,
        )
    }
}

impl RuleConfiguration for Bundler {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_required_properties(&properties, &["require-mode"])?;

        for (key, value) in properties {
            match key.as_str() {
                "modules-identifier" => match value {
                    RulePropertyValue::String(identifier) => {
                        self.modules_identifier = identifier;
                    }
                    _ => return Err(RuleConfigurationError::StringExpected(key)),
                },
                "require-mode" => match value {
                    RulePropertyValue::String(require_mode) => {
                        self.require_mode =
                            RequireMode::from_str(&require_mode).map_err(|err| {
                                RuleConfigurationError::UnexpectedValue {
                                    property: "require-mode".to_owned(),
                                    message: err,
                                }
                            })?;
                    }
                    RulePropertyValue::RequireMode(require_mode) => {
                        self.require_mode = require_mode;
                    }
                    _ => return Err(RuleConfigurationError::StringExpected(key)),
                },
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        BUNDLER_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        let mut properties = RuleProperties::new();

        properties.insert(
            "require-mode".to_owned(),
            RulePropertyValue::from(&self.require_mode),
        );

        if self.modules_identifier != DEFAULT_MODULE_IDENTIFIER {
            properties.insert(
                "modules-identifier".to_owned(),
                RulePropertyValue::from(&self.modules_identifier),
            );
        }

        properties
    }
}

const DEFAULT_MODULE_IDENTIFIER: &str = "__DARKLUA_BUNDLE_MODULES";

impl Default for Bundler {
    fn default() -> Self {
        Self {
            modules_identifier: DEFAULT_MODULE_IDENTIFIER.to_owned(),
            extra_module_relative_location: None,
            require_mode: RequireMode::default(),
            parser: Parser::default(),
        }
    }
}

#[inline]
fn default_module_folder_name() -> String {
    "init".to_owned()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PathRequireMode {
    #[serde(skip_serializing_if = "Option::is_none")]
    module_folder_name: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    sources: HashMap<String, PathBuf>,
}

impl PathRequireMode {
    pub fn new(module_folder_name: impl Into<String>) -> Self {
        Self {
            module_folder_name: Some(module_folder_name.into()),
            sources: HashMap::default(),
        }
    }

    pub(crate) fn module_folder_name(&self) -> String {
        self.module_folder_name
            .clone()
            .unwrap_or_else(default_module_folder_name)
    }

    pub(crate) fn get_source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "name")]
pub enum RequireMode {
    Path(PathRequireMode),
}

impl From<PathRequireMode> for RequireMode {
    fn from(mode: PathRequireMode) -> Self {
        Self::Path(mode)
    }
}

impl FromStr for RequireMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "path" => Self::Path(Default::default()),
            _ => return Err(format!("invalid require mode `{}`", s)),
        })
    }
}

impl Default for RequireMode {
    fn default() -> Self {
        Self::Path(Default::default())
    }
}

impl RequireMode {
    pub(crate) fn process_block<'a>(
        &self,
        block: &mut Block,
        source: PathBuf,
        extra_module_relative_location: Option<&'a Path>,
        module_identifier: &'a str,
        resources: &Resources,
        parser: &'a Parser,
    ) -> RuleProcessResult {
        match self {
            Self::Path(path_require_mode) => {
                let mut processor = RequirePathProcessor::new(
                    source,
                    extra_module_relative_location,
                    module_identifier,
                    path_require_mode,
                    resources,
                    parser,
                );
                DefaultVisitor::visit_block(block, &mut processor);
                if !processor.module_definitions.is_empty() {
                    for module_block in processor.module_definitions.drain(..).rev() {
                        block.insert_statement(0, DoStatement::new(module_block));
                    }
                    block.insert_statement(
                        0,
                        LocalAssignStatement::from_variable(module_identifier)
                            .with_value(TableExpression::default()),
                    );
                }
                processor.result()
            }
        }
    }
}

enum RequiredResource {
    Block(Block),
    Expression(Expression),
}

fn transcode<T: Serialize, E: Into<DarkluaError>>(
    value: Result<T, E>,
) -> Result<RequiredResource, DarkluaError> {
    let value = value.map_err(E::into)?;
    to_expression(&value)
        .map(RequiredResource::Expression)
        .map_err(DarkluaError::from)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> Bundler {
        Bundler::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_bundler", rule);
    }

    #[test]
    fn serialize_path_require_mode_with_custom_module_folder_name() {
        let rule: Box<dyn Rule> =
            Box::new(new_rule().with_require_mode(PathRequireMode::new("__init__")));

        assert_json_snapshot!("path_require_mode_with_custom_module_folder_name", rule);
    }

    #[test]
    fn serialize_path_require_mode_with_custom_module_folder_name_and_modules_identifier() {
        let rule: Box<dyn Rule> = Box::new(
            new_rule()
                .with_require_mode(PathRequireMode::new("__init__"))
                .with_modules_identifier("_CUSTOM_VAR"),
        );

        assert_json_snapshot!(
            "path_require_mode_with_custom_module_folder_name_and_modules_identifier",
            rule
        );
    }

    #[test]
    fn serialize_with_custom_modules_identifier() {
        let rule: Box<dyn Rule> = Box::new(new_rule().with_modules_identifier("_CUSTOM_VAR"));

        assert_json_snapshot!("custom_modules_identifier", rule);
    }
}
