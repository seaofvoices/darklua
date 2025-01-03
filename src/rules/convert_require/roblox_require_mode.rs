use serde::{Deserialize, Serialize};

use crate::{
    frontend::DarkluaResult,
    nodes::{Arguments, Expression, FieldExpression, FunctionCall, IndexExpression, Prefix},
    rules::{convert_require::rojo_sourcemap::RojoSourcemap, Context},
    utils, DarkluaError,
};

use std::path::{Component, Path, PathBuf};

use super::{
    instance_path::{get_parent_instance, script_identifier},
    RequireMode, RobloxIndexStyle,
};

#[derive(Deserialize)]
struct RojoTree {
    #[serde(rename = "$path")]
    path: PathBuf,
}

#[derive(Deserialize)]
struct RojoProject {
    tree: RojoTree,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct RobloxRequireMode {
    rojo_sourcemap: Option<PathBuf>,
    #[serde(default, deserialize_with = "crate::utils::string_or_struct")]
    indexing_style: RobloxIndexStyle,
    #[serde(skip)]
    cached_sourcemap: Option<RojoSourcemap>,
}

impl RobloxRequireMode {
    pub(crate) fn initialize(&mut self, context: &Context) -> DarkluaResult<()> {
        if let Some(ref rojo_sourcemap_path) = self
            .rojo_sourcemap
            .as_ref()
            .map(|rojo_sourcemap_path| context.project_location().join(rojo_sourcemap_path))
        {
            context.add_file_dependency(rojo_sourcemap_path.clone());

            let sourcemap_parent_location = get_relative_parent_path(rojo_sourcemap_path);
            let sourcemap = RojoSourcemap::parse(
                &context
                    .resources()
                    .get(rojo_sourcemap_path)
                    .map_err(|err| {
                        DarkluaError::from(err).context("while initializing Roblox require mode")
                    })?,
                sourcemap_parent_location,
            )
            .map_err(|err| {
                err.context(format!(
                    "unable to parse Rojo sourcemap at `{}`",
                    rojo_sourcemap_path.display()
                ))
            })?;
            self.cached_sourcemap = Some(sourcemap);
        }
        Ok(())
    }

    pub(crate) fn find_require(
        &self,
        call: &FunctionCall,
        context: &Context,
    ) -> DarkluaResult<Option<PathBuf>> {
        parse_roblox(call, context.current_path())
    }

    pub(crate) fn generate_require(
        &self,
        require_path: &Path,
        current: &RequireMode,
        context: &Context,
    ) -> DarkluaResult<Option<Arguments>> {
        let source_path = utils::normalize_path(context.current_path());
        log::trace!(
            "generate Roblox require for `{}` from `{}`",
            require_path.display(),
            source_path.display(),
        );

        if let Some((sourcemap, sourcemap_path)) = self
            .cached_sourcemap
            .as_ref()
            .zip(self.rojo_sourcemap.as_ref())
        {
            if let Some(require_relative_to_sourcemap) = get_relative_path(
                require_path,
                get_relative_parent_path(sourcemap_path),
                false,
            )? {
                log::trace!(
                    "  ⨽ use sourcemap at `{}` to find `{}`",
                    sourcemap_path.display(),
                    require_relative_to_sourcemap.display()
                );

                if let Some(instance_path) =
                    sourcemap.get_instance_path(&source_path, &require_relative_to_sourcemap)
                {
                    Ok(Some(Arguments::default().with_argument(
                        instance_path.convert(&self.indexing_style),
                    )))
                } else {
                    log::warn!(
                        "unable to find path `{}` in sourcemap (from `{}`)",
                        require_relative_to_sourcemap.display(),
                        source_path.display()
                    );
                    Ok(None)
                }
            } else {
                log::debug!(
                    "unable to get relative path from sourcemap for `{}`",
                    require_path.display()
                );
                Ok(None)
            }
        } else if let Some(relative_require_path) =
            get_relative_path(require_path, &source_path, true)?
        {
            log::trace!(
                "make require path relative to source: `{}`",
                relative_require_path.display()
            );

            let require_is_module_folder_name =
                current.is_module_folder_name(&relative_require_path);
            // if we are about to make a require to a path like `./x/y/z/init.lua`
            // we can pop the last component from the path
            let take_components = relative_require_path
                .components()
                .count()
                .saturating_sub(if require_is_module_folder_name { 1 } else { 0 });
            let mut path_components = relative_require_path.components().take(take_components);

            if let Some(first_component) = path_components.next() {
                let source_is_module_folder_name = current.is_module_folder_name(&source_path);

                let instance_path = path_components.try_fold(
                    match first_component {
                        Component::CurDir => {
                            if source_is_module_folder_name {
                                script_identifier().into()
                            } else {
                                get_parent_instance(script_identifier())
                            }
                        }
                        Component::ParentDir => {
                            if source_is_module_folder_name {
                                get_parent_instance(script_identifier())
                            } else {
                                get_parent_instance(get_parent_instance(script_identifier()))
                            }
                        }
                        Component::Normal(_) => {
                            return Err(DarkluaError::custom(format!(
                                concat!(
                                    "unable to convert path `{}`: the require path should be ",
                                    "relative and start with `.` or `..` (got `{}`)"
                                ),
                                require_path.display(),
                                relative_require_path.display(),
                            )))
                        }
                        Component::Prefix(_) | Component::RootDir => {
                            return Err(DarkluaError::custom(format!(
                                concat!(
                                    "unable to convert absolute path `{}`: ",
                                    "without a provided Rojo sourcemap, ",
                                    "darklua can only convert relative paths ",
                                    "(starting with `.` or `..`)"
                                ),
                                require_path.display(),
                            )))
                        }
                    },
                    |instance: Prefix, component| match component {
                        Component::CurDir => Ok(instance),
                        Component::ParentDir => Ok(get_parent_instance(instance)),
                        Component::Normal(name) => utils::convert_os_string(name)
                            .map(|child_name| self.indexing_style.index(instance, child_name)),
                        Component::Prefix(_) | Component::RootDir => {
                            Err(DarkluaError::custom(format!(
                                "unable to convert path `{}`: unexpected component in relative path `{}`",
                                require_path.display(),
                                relative_require_path.display(),
                            )))
                        },
                    },
                )?;

                Ok(Some(Arguments::default().with_argument(instance_path)))
            } else {
                Err(DarkluaError::custom(format!(
                    "unable to convert path `{}` from `{}` without a sourcemap: the relative path is empty `{}`",
                    require_path.display(),
                    source_path.display(),
                    relative_require_path.display(),
                )))
            }
        } else {
            Err(DarkluaError::custom(format!(
                concat!(
                    "unable to convert path `{}` from `{}` without a sourcemap: unable to ",
                    "make the require path relative to the source file"
                ),
                require_path.display(),
                source_path.display(),
            )))
        }
    }
}

fn get_relative_path(
    require_path: &Path,
    source_path: &Path,
    use_current_dir_prefix: bool,
) -> Result<Option<PathBuf>, DarkluaError> {
    Ok(
        pathdiff::diff_paths(require_path, get_relative_parent_path(source_path))
            .map(|path| {
                if use_current_dir_prefix && !path.starts_with(".") && !path.starts_with("..") {
                    Path::new(".").join(path)
                } else if !use_current_dir_prefix && path.starts_with(".") {
                    path.strip_prefix(".")
                        .map(Path::to_path_buf)
                        .ok()
                        .unwrap_or(path)
                } else {
                    path
                }
            })
            .map(utils::normalize_path_with_current_dir),
    )
}

fn get_relative_parent_path(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) => {
            if parent == Path::new("") {
                Path::new(".")
            } else {
                parent
            }
        }
        None => Path::new(".."),
    }
}

pub fn parse_roblox(call: &FunctionCall, starting_path: &Path) -> DarkluaResult<Option<PathBuf>> {
    let Arguments::Tuple(args) = call.get_arguments() else {
        Err(
            DarkluaError::custom("unexpected require call, only accepts tuples")
                .context("while finding roblox requires"),
        )?
    };

    let mut path_builder = Vec::<String>::new();
    let mut current_path = starting_path.to_path_buf();
    let mut parented = false;

    match args.iter_values().next() {
        Some(Expression::Field(field)) => {
            parse_roblox_field(field, &mut path_builder, &mut current_path, &mut parented)?
        }
        Some(Expression::Index(index)) => {
            parse_roblox_index(index, &mut path_builder, &mut current_path, &mut parented)?
        }
        _ => Err(DarkluaError::custom(
            "unexpected require argument, only accepts fields or indexes",
        )
        .context("while getting roblox path"))?,
    };

    if let Some(back) = path_builder.last() {
        if back != "script" {
            Err(
                DarkluaError::custom("roblox requires must start with `script.`")
                    .context("while getting roblox require path"),
            )?
        } else {
            path_builder.pop();
        }
    }

    while let Some(x) = path_builder.pop() {
        current_path.push(x)
    }

    let mut base_path = starting_path.to_path_buf();
    base_path.pop();

    let mut project_json = current_path.clone();
    project_json.push("default.project.json");
    if let Ok(project_json) = std::fs::File::open(project_json) {
        let project_data: RojoProject = serde_json::from_reader(project_json)?;
        current_path.push(project_data.tree.path)
    }

    Ok(Some(current_path))
}

fn parse_roblox_prefix(
    prefix: &Prefix,
    path_builder: &mut Vec<String>,
    current_path: &mut PathBuf,
    parented: &mut bool,
) -> DarkluaResult<()> {
    match prefix {
        Prefix::Field(x) => parse_roblox_field(x, path_builder, current_path, parented)?,
        Prefix::Index(x) => parse_roblox_index(x, path_builder, current_path, parented)?,
        Prefix::Identifier(x) => {
            handle_roblox_script_parent(x.get_name(), path_builder, current_path, parented)?
        }
        _ => Err(
            DarkluaError::custom("unexpected prefix, only constants accepted")
                .context("while parsing roblox require"),
        )?,
    };
    Ok(())
}

fn parse_roblox_expression(
    expression: &Expression,
    path_builder: &mut Vec<String>,
    current_path: &mut PathBuf,
    parented: &mut bool,
) -> DarkluaResult<()> {
    match expression {
        Expression::Field(x) => parse_roblox_field(x, path_builder, current_path, parented)?,
        Expression::Index(x) => parse_roblox_index(x, path_builder, current_path, parented)?,
        Expression::Identifier(x) => {
            handle_roblox_script_parent(x.get_name(), path_builder, current_path, parented)?
        }
        Expression::String(x) => {
            handle_roblox_script_parent(x.get_value(), path_builder, current_path, parented)?
        }
        _ => Err(
            DarkluaError::custom("unexpected expression, only constants accepted")
                .context("while parsing roblox require"),
        )?,
    };
    Ok(())
}

fn parse_roblox_field(
    field: &FieldExpression,
    path_builder: &mut Vec<String>,
    current_path: &mut PathBuf,
    parented: &mut bool,
) -> DarkluaResult<()> {
    handle_roblox_script_parent(
        field.get_field().get_name(),
        path_builder,
        current_path,
        parented,
    )?;
    parse_roblox_prefix(field.get_prefix(), path_builder, current_path, parented)
}

fn parse_roblox_index(
    index: &IndexExpression,
    path_builder: &mut Vec<String>,
    current_path: &mut PathBuf,
    parented: &mut bool,
) -> DarkluaResult<()> {
    parse_roblox_expression(index.get_index(), path_builder, current_path, parented)?;
    parse_roblox_prefix(index.get_prefix(), path_builder, current_path, parented)
}

fn handle_roblox_script_parent(
    str: &str,
    path_builder: &mut Vec<String>,
    current_path: &mut PathBuf,
    parented: &mut bool,
) -> DarkluaResult<()> {
    match str {
        "script" => {
            if let Some(back) = path_builder.last() {
                if !(*parented) {
                    current_path.pop();
                    *parented = true;
                }

                if back == "Parent" {
                    path_builder.pop();
                }
            }
        }
        "Parent" => {
            *parented = true;
            current_path.pop();
        }
        _ => {}
    };
    path_builder.push(str.to_string());
    Ok(())
}
