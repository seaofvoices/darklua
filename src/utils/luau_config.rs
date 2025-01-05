use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cached::proc_macro::cached;
use serde::Deserialize;

use crate::{DarkluaError, Resources};

const LUAU_RC_FILE_NAME: &str = ".luaurc";

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct LuauConfiguration {
    pub(crate) aliases: HashMap<String, PathBuf>,
}

#[cached(
    key = "Option<PathBuf>",
    convert = r##"{ luau_file.parent().map(Path::to_path_buf) }"##,
    result = true
)]
pub(crate) fn find_luau_configuration(
    luau_file: &Path,
    resources: &Resources,
) -> Result<Option<LuauConfiguration>, DarkluaError> {
    for ancestor in luau_file.ancestors() {
        let possible_config = ancestor.join(LUAU_RC_FILE_NAME);

        if resources.exists(&possible_config)? {
            let config = resources.get(&possible_config)?;

            return serde_json::from_str(&config)
                .map(|mut config: LuauConfiguration| {
                    config.aliases = config
                        .aliases
                        .into_iter()
                        .map(|(mut key, value)| {
                            key.insert(0, '@');
                            (key, value)
                        })
                        .collect();

                    Some(config)
                })
                .map_err(Into::into);
        }
    }

    Ok(None)
}
