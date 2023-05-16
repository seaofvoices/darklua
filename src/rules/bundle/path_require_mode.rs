use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use wax::{Glob, Pattern};

use crate::frontend::DarkluaResult;
use crate::nodes::{
    Arguments, AssignStatement, Block, DoStatement, Expression, FieldExpression, FunctionCall,
    Identifier, LastStatement, LocalAssignStatement, Prefix, Statement, TableExpression,
};
use crate::process::utils::{generate_identifier, identifier_permutator, CharPermutator};
use crate::process::{DefaultVisitor, IdentifierTracker, NodeProcessor, NodeVisitor, ScopeVisitor};
use crate::rules::{
    ContextBuilder, ReplaceReferencedTokens, Rule, RuleConfiguration, RuleProcessResult,
};
use crate::{utils, DarkluaError, Parser, Resources};

use super::expression_serializer::to_expression;

enum RequiredResource {
    Block(Block),
    Expression(Expression),
}

#[derive(Debug)]
struct RequirePathLocator<'a, 'b> {
    path_require_mode: &'a PathRequireMode,
    extra_module_relative_location: Option<&'a Path>,
    resources: &'b Resources,
    cached_exclude_globs: wax::Any<'a>,
}

impl<'a, 'b> RequirePathLocator<'a, 'b> {
    fn new(
        path_require_mode: &'a PathRequireMode,
        extra_module_relative_location: Option<&'a Path>,
        resources: &'b Resources,
    ) -> Self {
        Self {
            path_require_mode,
            extra_module_relative_location,
            resources,
            cached_exclude_globs: wax::any::<Glob, _>(path_require_mode.excludes().filter_map(
                |exclusion| match Glob::new(exclusion) {
                    Ok(glob) => Some(glob),
                    Err(err) => {
                        log::warn!(
                            "unable to create exclude matcher from `{}`: {}",
                            exclusion,
                            err.to_string()
                        );
                        None
                    }
                },
            ))
            .expect("exclude globs errors should be filtered and only emit a warning"),
        }
    }

    fn normalize_require_path(
        &self,
        path: impl Into<PathBuf>,
        source: &Path,
    ) -> Result<PathBuf, DarkluaError> {
        let mut path: PathBuf = path.into();
        if is_require_relative(&path) {
            if self.resources.is_file(source)? {
                let mut new_path = source.to_path_buf();
                new_path.pop();
                new_path.push(path);
                path = new_path;
            } else {
                path = source.join(path);
            }
        } else if !path.is_absolute() {
            let extra_module_relative_location =
                self.extra_module_relative_location.ok_or_else(|| {
                    DarkluaError::invalid_resource_path(
                        path.display().to_string(),
                        "unable to obtain configuration file location",
                    )
                })?;

            let mut components = path.components();
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

        Ok(utils::normalize_path_with_current_dir(path))
    }

    fn is_excluded(&self, require_path: &Path) -> bool {
        let excluded = self.cached_exclude_globs.is_match(require_path);
        log::trace!(
            "verify if `{}` is excluded: {}",
            require_path.display(),
            excluded
        );
        excluded
    }
}

// the `is_relative` method from std::path::Path is not what darklua needs
// to consider a require relative, which is paths that starts with `.` or `..`
fn is_require_relative(path: &Path) -> bool {
    path.starts_with(Path::new(".")) || path.starts_with(Path::new(".."))
}

const REQUIRE_FUNCTION_IDENTIFIER: &str = "require";

#[derive(Debug)]
struct RequirePathProcessor<'a, 'b> {
    identifier_tracker: IdentifierTracker,
    path_locator: RequirePathLocator<'a, 'b>,
    module_definitions: BuildModuleDefinitions<'a>,
    source: PathBuf,
    module_cache: HashMap<PathBuf, Expression>,
    require_stack: Vec<PathBuf>,
    skip_module_paths: HashSet<PathBuf>,
    parser: &'a Parser,
    resources: &'b Resources,
    errors: Vec<String>,
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
            identifier_tracker: IdentifierTracker::new(),
            path_locator: RequirePathLocator::new(
                path_require_mode,
                extra_module_relative_location,
                resources,
            ),
            module_definitions: BuildModuleDefinitions::new(modules_identifier),
            source: source.into(),
            module_cache: Default::default(),
            require_stack: Default::default(),
            skip_module_paths: Default::default(),
            resources,
            parser,
            errors: Vec::new(),
        }
    }

    fn apply(self, block: &mut Block) -> RuleProcessResult {
        self.module_definitions.apply(block);
        match self.errors.len() {
            0 => Ok(()),
            1 => Err(self.errors.first().unwrap().to_string()),
            _ => Err(format!("- {}", self.errors.join("\n- "))),
        }
    }

    fn require_call(&self, call: &FunctionCall) -> Option<PathBuf> {
        if call.get_method().is_some() {
            return None;
        }

        match call.get_prefix() {
            Prefix::Identifier(identifier)
                if identifier.get_name() == REQUIRE_FUNCTION_IDENTIFIER =>
            {
                if self
                    .identifier_tracker
                    .is_identifier_used(REQUIRE_FUNCTION_IDENTIFIER)
                {
                    return None;
                }
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
        .map(utils::normalize_path_with_current_dir)
    }

    fn try_inline_call(&mut self, call: &FunctionCall) -> Option<Expression> {
        let literal_require_path = self.require_call(call)?;

        if self.path_locator.is_excluded(&literal_require_path) {
            return None;
        }

        let require_path = match self
            .path_locator
            .normalize_require_path(&literal_require_path, &self.source)
        {
            Ok(path) => path,
            Err(err) => {
                self.errors.push(err.to_string());
                return None;
            }
        };

        log::debug!(
            "found require call to path `{}` (normalized `{}`)",
            literal_require_path.display(),
            require_path.display()
        );

        if self.skip_module_paths.contains(&require_path) {
            return None;
        }

        match self.inline_require(&require_path) {
            Ok(expression) => Some(expression),
            Err(error) => {
                self.errors.push(error.to_string());
                self.skip_module_paths.insert(require_path);
                None
            }
        }
    }

    fn inline_require(&mut self, require_path: &Path) -> DarkluaResult<Expression> {
        if let Some(expression) = self.module_cache.get(require_path) {
            Ok(expression.clone())
        } else {
            if let Some(i) = self
                .require_stack
                .iter()
                .enumerate()
                .find(|(_, path)| **path == require_path)
                .map(|(i, _)| i)
            {
                let require_stack_paths: Vec<_> = self
                    .require_stack
                    .iter()
                    .skip(i)
                    .map(|path| path.display().to_string())
                    .chain(std::iter::once(require_path.display().to_string()))
                    .collect();

                return Err(DarkluaError::custom(format!(
                    "cyclic require detected with `{}`",
                    require_stack_paths.join("` > `")
                )));
            }

            self.require_stack.push(require_path.to_path_buf());
            let required_resource = self.require_resource(require_path);
            self.require_stack.pop();

            let module_value = self
                .module_definitions
                .build_module_from_resource(required_resource?, require_path)?;

            self.module_cache
                .insert(require_path.to_path_buf(), module_value.clone());

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
                "json" | "json5" => {
                    log::debug!("transcode json data to Lua from `{}`", path.display());
                    transcode(json5::from_str::<serde_json::Value>(&content))
                }
                "yml" | "yaml" => {
                    log::debug!("transcode yaml data to Lua from `{}`", path.display());
                    transcode(serde_yaml::from_str::<serde_yaml::Value>(&content))
                }
                "toml" => {
                    log::debug!("transcode toml data to Lua from `{}`", path.display());
                    transcode(toml::from_str::<toml::Value>(&content))
                }
                _ => Err(DarkluaError::invalid_resource_extension(path)),
            },
            None => unreachable!("extension should be defined"),
        }
    }
}

impl<'a, 'b> Deref for RequirePathProcessor<'a, 'b> {
    type Target = IdentifierTracker;

    fn deref(&self) -> &Self::Target {
        &self.identifier_tracker
    }
}

impl<'a, 'b> DerefMut for RequirePathProcessor<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identifier_tracker
    }
}

fn transcode<T: Serialize, E: Into<DarkluaError>>(
    value: Result<T, E>,
) -> Result<RequiredResource, DarkluaError> {
    let value = value.map_err(E::into)?;
    to_expression(&value)
        .map(RequiredResource::Expression)
        .map_err(DarkluaError::from)
}

impl<'a, 'b> NodeProcessor for RequirePathProcessor<'a, 'b> {
    fn process_expression(&mut self, expression: &mut Expression) {
        if let Expression::Call(call) = expression {
            if let Some(replace_with) = self.try_inline_call(call) {
                *expression = replace_with;
            }
        }
    }

    fn process_prefix_expression(&mut self, prefix: &mut Prefix) {
        if let Prefix::Call(call) = prefix {
            if let Some(replace_with) = self.try_inline_call(call) {
                *prefix = replace_with.into();
            }
        }
    }

    fn process_statement(&mut self, statement: &mut Statement) {
        if let Statement::Call(call) = statement {
            if let Some(replace_with) = self.try_inline_call(call) {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PathRequireMode {
    #[serde(skip_serializing_if = "Option::is_none")]
    module_folder_name: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    sources: HashMap<String, PathBuf>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    excludes: HashSet<String>,
}

impl PathRequireMode {
    pub fn new(module_folder_name: impl Into<String>) -> Self {
        Self {
            module_folder_name: Some(module_folder_name.into()),
            sources: Default::default(),
            excludes: Default::default(),
        }
    }

    pub fn with_exclude(mut self, exclude: impl Into<String>) -> Self {
        self.excludes.insert(exclude.into());
        self
    }

    pub(crate) fn module_folder_name(&self) -> String {
        self.module_folder_name
            .clone()
            .unwrap_or_else(|| "init".to_owned())
    }

    pub(crate) fn get_source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }

    pub(crate) fn excludes(&self) -> impl Iterator<Item = &str> {
        self.excludes.iter().map(AsRef::as_ref)
    }

    pub(crate) fn process_block<'a>(
        &self,
        block: &mut Block,
        source: PathBuf,
        extra_module_relative_location: Option<&'a Path>,
        module_identifier: &'a str,
        resources: &Resources,
        parser: &'a Parser,
    ) -> RuleProcessResult {
        let mut processor = RequirePathProcessor::new(
            source,
            extra_module_relative_location,
            module_identifier,
            self,
            resources,
            parser,
        );
        ScopeVisitor::visit_block(block, &mut processor);
        processor.apply(block)
    }
}

#[derive(Debug)]
struct BuildModuleDefinitions<'a> {
    modules_identifier: &'a str,
    module_definitions: Vec<Block>,
    module_name_permutator: CharPermutator,
}

impl<'a> BuildModuleDefinitions<'a> {
    fn new(modules_identifier: &'a str) -> Self {
        Self {
            modules_identifier,
            module_definitions: Vec::new(),
            module_name_permutator: identifier_permutator(),
        }
    }

    fn build_module_from_resource(
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

    fn apply(mut self, block: &mut Block) {
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
