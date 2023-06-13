mod module_definitions;
mod path_iterator;

use module_definitions::BuildModuleDefinitions;

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{iter, mem};

use serde::{Deserialize, Serialize};

use crate::frontend::DarkluaResult;
use crate::nodes::{
    Arguments, Block, DoStatement, Expression, FunctionCall, LocalAssignStatement, Prefix,
    Statement,
};
use crate::process::{DefaultVisitor, IdentifierTracker, NodeProcessor, NodeVisitor, ScopeVisitor};
use crate::rules::{
    Context, ContextBuilder, FlawlessRule, ReplaceReferencedTokens, RuleProcessResult,
};
use crate::utils::Timer;
use crate::{utils, DarkluaError, Resources};

use super::expression_serializer::to_expression;
use super::BundleOptions;

pub(crate) enum RequiredResource {
    Block(Block),
    Expression(Expression),
}

#[derive(Debug)]
struct RequirePathLocator<'a, 'b> {
    path_require_mode: &'a PathRequireMode,
    extra_module_relative_location: &'a Path,
    resources: &'b Resources,
}

impl<'a, 'b> RequirePathLocator<'a, 'b> {
    fn new(
        path_require_mode: &'a PathRequireMode,
        extra_module_relative_location: &'a Path,
        resources: &'b Resources,
    ) -> Self {
        Self {
            path_require_mode,
            extra_module_relative_location,
            resources,
        }
    }

    fn find_require_path(
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

            let mut extra_module_location = self.extra_module_relative_location.to_path_buf();
            extra_module_location.push(self.path_require_mode.get_source(source_name).ok_or_else(
                || {
                    DarkluaError::invalid_resource_path(
                        path.display().to_string(),
                        format!("unknown source name `{}`", source_name),
                    )
                },
            )?);
            extra_module_location.extend(components);
            path = extra_module_location;
        }

        let normalized_path = utils::normalize_path_with_current_dir(&path);
        for potential_path in path_iterator::find_require_paths(
            &normalized_path,
            &self.path_require_mode.module_folder_name(),
        ) {
            if self.resources.is_file(&potential_path)? {
                return Ok(utils::normalize_path_with_current_dir(potential_path));
            }
        }

        Err(
            DarkluaError::resource_not_found(&normalized_path).context(format!(
                "tried `{}`",
                path_iterator::find_require_paths(
                    &normalized_path,
                    &self.path_require_mode.module_folder_name(),
                )
                .map(|potential_path| potential_path.display().to_string())
                .collect::<Vec<_>>()
                .join("`, `")
            )),
        )
    }
}

// the `is_relative` method from std::path::Path is not what darklua needs
// to consider a requi re relative, which is paths that starts with `.` or `..`
fn is_require_relative(path: &Path) -> bool {
    path.starts_with(Path::new(".")) || path.starts_with(Path::new(".."))
}

const REQUIRE_FUNCTION_IDENTIFIER: &str = "require";

#[derive(Debug)]
struct RequirePathProcessor<'a, 'b> {
    options: &'a BundleOptions,
    identifier_tracker: IdentifierTracker,
    path_locator: RequirePathLocator<'a, 'b>,
    module_definitions: BuildModuleDefinitions<'a>,
    source: PathBuf,
    module_cache: HashMap<PathBuf, Expression>,
    require_stack: Vec<PathBuf>,
    skip_module_paths: HashSet<PathBuf>,
    resources: &'b Resources,
    errors: Vec<String>,
}

impl<'a, 'b> RequirePathProcessor<'a, 'b> {
    fn new(
        source: impl Into<PathBuf>,
        options: &'a BundleOptions,
        path_require_mode: &'a PathRequireMode,
        resources: &'b Resources,
    ) -> Self {
        Self {
            options,
            identifier_tracker: IdentifierTracker::new(),
            path_locator: RequirePathLocator::new(
                path_require_mode,
                options.extra_module_relative_location(),
                resources,
            ),
            module_definitions: BuildModuleDefinitions::new(options.modules_identifier()),
            source: source.into(),
            module_cache: Default::default(),
            require_stack: Default::default(),
            skip_module_paths: Default::default(),
            resources,
            errors: Vec::new(),
        }
    }

    fn apply(self, block: &mut Block, context: &Context) -> RuleProcessResult {
        self.module_definitions.apply(block, context);
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

        if self.options.is_excluded(&literal_require_path) {
            log::info!(
                "exclude `{}` from bundle [from `{}`]",
                literal_require_path.display(),
                self.source.display()
            );
            return None;
        }

        let require_path = match self
            .path_locator
            .find_require_path(&literal_require_path, &self.source)
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
            log::trace!(
                "skip `{}` because it previously errored",
                require_path.display()
            );
            return None;
        }

        match self.inline_require(&require_path, call) {
            Ok(expression) => Some(expression),
            Err(error) => {
                self.errors.push(error.to_string());
                self.skip_module_paths.insert(require_path);
                None
            }
        }
    }

    fn inline_require(
        &mut self,
        require_path: &Path,
        call: &FunctionCall,
    ) -> DarkluaResult<Expression> {
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
                    .chain(iter::once(require_path.display().to_string()))
                    .collect();

                return Err(DarkluaError::custom(format!(
                    "cyclic require detected with `{}`",
                    require_stack_paths.join("` > `")
                )));
            }

            self.require_stack.push(require_path.to_path_buf());
            let required_resource = self.require_resource(require_path);
            self.require_stack.pop();

            let module_value = self.module_definitions.build_module_from_resource(
                required_resource?,
                require_path,
                call,
            )?;

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
                    let parser_timer = Timer::now();
                    let mut block =
                        self.options
                            .parser()
                            .parse(&content)
                            .map_err(|parser_error| {
                                DarkluaError::parser_error(path.to_path_buf(), parser_error)
                            })?;
                    log::debug!(
                        "parsed `{}` in {}",
                        path.display(),
                        parser_timer.duration_label()
                    );

                    let path_buf = path.to_path_buf();
                    let current_source = mem::replace(&mut self.source, path.to_path_buf());

                    let apply_processor_timer = Timer::now();
                    DefaultVisitor::visit_block(&mut block, self);
                    log::debug!(
                        "processed `{}` into bundle in {}",
                        path_buf.display(),
                        apply_processor_timer.duration_label()
                    );

                    self.source = current_source;

                    if self.options.parser().is_preserving_tokens() {
                        let context =
                            ContextBuilder::new(path_buf.clone(), self.resources, &content).build();
                        // run `replace_referenced_tokens` rule to avoid generating invalid code
                        // when using the token-based generator
                        let replace_tokens = ReplaceReferencedTokens::default();

                        let apply_replace_tokens_timer = Timer::now();

                        replace_tokens.flawless_process(&mut block, &context);

                        log::trace!(
                            "replaced token references for `{}` in {}",
                            path_buf.display(),
                            apply_replace_tokens_timer.duration_label()
                        );
                    }
                    Ok(RequiredResource::Block(block))
                }
                "json" | "json5" => {
                    transcode("json", path, json5::from_str::<serde_json::Value>, &content)
                }
                "yml" | "yaml" => transcode(
                    "yaml",
                    path,
                    serde_yaml::from_str::<serde_yaml::Value>,
                    &content,
                ),
                "toml" => transcode("toml", path, toml::from_str::<toml::Value>, &content),
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

fn transcode<'a, T, E>(
    label: &'static str,
    path: &Path,
    deserialize_value: impl Fn(&'a str) -> Result<T, E>,
    content: &'a str,
) -> Result<RequiredResource, DarkluaError>
where
    T: Serialize,
    E: Into<DarkluaError>,
{
    log::trace!("transcode {} data to Lua from `{}`", label, path.display());
    let transcode_duration = Timer::now();
    let value = deserialize_value(content).map_err(E::into)?;
    let expression = to_expression(&value)
        .map(RequiredResource::Expression)
        .map_err(DarkluaError::from);
    log::debug!(
        "transcoded {} data to Lua from `{}` in {}",
        label,
        path.display(),
        transcode_duration.duration_label()
    );
    expression
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
}

impl PathRequireMode {
    pub fn new(module_folder_name: impl Into<String>) -> Self {
        Self {
            module_folder_name: Some(module_folder_name.into()),
            sources: Default::default(),
        }
    }

    pub(crate) fn module_folder_name(&self) -> String {
        self.module_folder_name
            .clone()
            .unwrap_or_else(|| "init".to_owned())
    }

    pub(crate) fn get_source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }

    pub(crate) fn process_block(
        &self,
        block: &mut Block,
        context: &Context,
        options: &BundleOptions,
    ) -> RuleProcessResult {
        let mut processor =
            RequirePathProcessor::new(context.current_path(), options, self, context.resources());
        ScopeVisitor::visit_block(block, &mut processor);
        processor.apply(block, context)
    }
}
