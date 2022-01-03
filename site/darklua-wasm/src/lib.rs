mod utils;

use darklua_core::{
    generator::{LuaGenerator, TokenBasedLuaGenerator},
    nodes::Block,
    rules::{self, get_default_rules, Context, Rule},
    Parser,
};
use serde::{Deserialize, Serialize};
use utils::set_panic_hook;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "get_default_rules")]
    pub rules: Vec<Box<dyn Rule>>,
}

fn generate_code(original_code: &str, block: &Block) -> String {
    let mut generator = TokenBasedLuaGenerator::new(original_code);
    generator.write_block(block);
    generator.into_string()
}

#[wasm_bindgen]
pub fn process_code(code: &str, opt_config: JsValue) -> Result<String, JsValue> {
    set_panic_hook();

    let config = if opt_config.is_undefined() {
        Config {
            rules: get_default_rules(),
        }
    } else if opt_config.is_object() {
        let config_string = String::from(js_sys::JSON::stringify(&opt_config)?);
        json5::from_str(&config_string)
            .map_err(|err| format!("unable to parse configuration: {}", err))?
    } else {
        opt_config
            .as_string()
            .ok_or_else(|| format!("unsupported type passed as configuration"))
            .and_then(|config| {
                json5::from_str(&config)
                    .map_err(|err| format!("unable to parse configuration: {}", err))
            })?
    };

    let parser = Parser::default().preserve_tokens();

    let mut block = parser.parse(code).map_err(|error| error.to_string())?;

    for (index, rule) in config.rules.iter().enumerate() {
        let mut context = Context::default();
        rule.process(&mut block, &mut context)
            .map_err(|rule_errors| {
                let errors: Vec<_> = rule_errors.iter().map(ToString::to_string).collect();
                format!(
                    "error with rule {} ({}):\n -> {}",
                    rule.get_name().to_owned(),
                    index,
                    errors.join("\n -> "),
                )
            })?;
    }

    let lua_code = generate_code(&code, &block);

    Ok(lua_code)
}

#[wasm_bindgen]
pub fn get_all_rule_names() -> Box<[JsValue]> {
    rules::get_all_rule_names()
        .into_iter()
        .map(JsValue::from_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
