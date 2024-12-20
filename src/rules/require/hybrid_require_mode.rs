use std::{collections::{HashMap, VecDeque}, path::{Path, PathBuf}};

use serde::{Serialize, Deserialize};

use crate::{frontend::DarkluaResult, nodes::{Arguments, Expression, FieldExpression, FunctionCall, Prefix}, rules::{parse_roblox, require::path_require_mode::{get_default_module_folder_name, is_default_module_folder_name}}, DarkluaError};

use super::{match_path_require_call, RequirePathLocatorMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HybridRequireMode {
    #[serde(
        skip_serializing_if = "is_default_module_folder_name",
        default = "get_default_module_folder_name"
    )]
    module_folder_name: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    sources: HashMap<String, PathBuf>,
    #[serde(default)]
    convert_ts_imports: bool,
}

impl RequirePathLocatorMode for HybridRequireMode {
    fn get_source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }
    fn module_folder_name(&self) -> &str {
        &self.module_folder_name
    }
    fn match_path_require_call(&self, call: &FunctionCall, source: &Path) -> Option<PathBuf> {
       parse_roblox(call, source).ok().flatten().or(match_path_require_call(call))
    }
    fn require_call(&self, call: &FunctionCall, source: &Path) -> Option<PathBuf> {
        if !self.convert_ts_imports {
            return None
        }

        let Prefix::Field(field) = call.get_prefix() else { return None };
        match field.get_prefix() {
            Prefix::Identifier(x) if x.get_name() == "TS" && x.get_token().is_none() => Some(()),
            _ => None
        }?;
        if !(field.get_field().get_name() == "import" && field.get_field().get_token().is_none()) {
            return None
        }

        let Arguments::Tuple(values) = call.get_arguments() else { return None };
        let mut current_path = source.to_path_buf();

        if current_path.ends_with("init.lua") {
            current_path.pop();
        }

        let mut path_builder = VecDeque::new();

        values.iter_values().for_each(|v| { parse_roblox_expression(v, &mut path_builder, &mut current_path).ok(); });

        while let Some(x) = path_builder.pop_back() {
            current_path.push(x);
        }

        pathdiff::diff_paths(current_path, PathBuf::from("./"))
    }
}

fn parse_roblox_call(call: &FunctionCall, current_path: &mut PathBuf) -> DarkluaResult<()> {
    match call.get_prefix() {
        Prefix::Field(field) => {
            match field.get_prefix() {
                Prefix::Identifier(x) if x.get_name() == "TS" && x.get_token().is_none() => {},
                _ => return Err(DarkluaError::custom("expected call to be apart of the TS module").context("while parsing roblox-ts require"))?
            };
            if !(field.get_field().get_name() == "getModule" && field.get_field().get_token().is_none()) {
                return Err(DarkluaError::custom("expected call to be TS.getModule").context("while parsing roblox-ts require"));
            }
        },
        _ => return Err(DarkluaError::custom("a"))?
    };
    
    let mut temp_path = PathBuf::from("node_modules");
    let Arguments::Tuple(args) = call.get_arguments() else { return Err(DarkluaError::custom("expected call arguments for TS.getModule to be a tuple").context("while parsing roblox-ts require"))? };
    args.iter_values().for_each(|arg| {
        match arg {
            Expression::String(x) => temp_path.push(x.get_value().to_string()),
            _ => {},
        }
    });

    let _ = temp_path.join(&current_path);
    *current_path = temp_path;

    Ok(())
}

fn parse_roblox_prefix(prefix: &Prefix, path_builder: &mut VecDeque<String>, current_path: &mut PathBuf) -> DarkluaResult<()> {
    match prefix {
        Prefix::Field(x) => parse_roblox_field(&x, path_builder, current_path)?,
        Prefix::Identifier(x) => handle_roblox_script_parent(&x.get_name(), path_builder, current_path)?,
        Prefix::Call(x) => parse_roblox_call(x, current_path)?,
        _ => Err(DarkluaError::custom("unexpected prefix, only constants accepted").context("while parsing roblox require"))?
    };
    Ok(())
}

fn parse_roblox_expression(expression: &Expression, path_builder: &mut VecDeque<String>, current_path: &mut PathBuf) -> DarkluaResult<()> {
    match expression {
        Expression::Field(x) => parse_roblox_field(x, path_builder, current_path)?,
        Expression::Identifier(x) => handle_roblox_script_parent(&x.get_name(), path_builder, current_path)?,
        Expression::String(x) => handle_roblox_script_parent(x.get_value(), path_builder, current_path)?,
        Expression::Call(x) => parse_roblox_call(x, current_path)?,
        _ => Err(DarkluaError::custom("unexpected expression, only constants accepted").context("while parsing roblox require"))?,
    };
    Ok(())
}

fn parse_roblox_field(field: &Box<FieldExpression>, path_builder: &mut VecDeque<String>, current_path: &mut PathBuf) -> DarkluaResult<()> {
    parse_roblox_prefix(field.get_prefix(), path_builder, current_path)?;
    handle_roblox_script_parent(&field.get_field().get_name(), path_builder, current_path)
}

fn handle_roblox_script_parent(str: &str, path_builder: &mut VecDeque<String>, current_path: &mut PathBuf) -> DarkluaResult<()> {
    match str {
        "script" => {},
        "Parent" => { current_path.pop(); },
        x => path_builder.push_front(x.to_string()),
    };
    Ok(())
}
