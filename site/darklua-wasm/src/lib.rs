mod utils;

use darklua_core::{Configuration, Options, Resources};
use utils::set_panic_hook;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn process_code(code: &str, opt_config: JsValue) -> Result<String, JsValue> {
    set_panic_hook();

    let config = if opt_config.is_undefined() {
        Configuration::default()
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

    let resources = Resources::from_memory();
    const LOCATION: &str = "file.lua";
    resources.write(LOCATION, code).unwrap();

    let result = darklua_core::process(
        &resources,
        Options::new(LOCATION).with_configuration(config),
    );

    match result.result() {
        Ok(()) => {
            let lua_code = resources.get(LOCATION).unwrap();

            Ok(lua_code)
        }
        Err(errors) => {
            let errors: Vec<_> = errors
                .into_iter()
                .map(|error| format!("-> {}", error))
                .collect();
            Err(format!("unable to process code:\n{}", errors.join("\n")).into())
        }
    }
}

#[wasm_bindgen]
pub fn get_all_rule_names() -> Box<[JsValue]> {
    darklua_core::rules::get_all_rule_names()
        .into_iter()
        .map(JsValue::from_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
