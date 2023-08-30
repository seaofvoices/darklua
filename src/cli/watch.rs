use super::{process::{Options, self}, GlobalOptions, CommandResult};

use std::sync::Arc;
use notify::{Watcher, RecursiveMode};

pub fn run(options: &Options, global: &GlobalOptions) -> CommandResult {
    let mut watch_path = options.input_path.clone();
    watch_path.pop();

    let options = Arc::new(options.clone());
    let global = Arc::new(global.clone());

    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(event) => {
                if event.kind.is_create() || event.kind.is_modify() {
                    let Err(e) = process::run(&*(options.clone()), &*(global.clone())) else { return };
                    log::error!("there was an process error while attempting to bundle: {:?}", e);
                }
            }
            Err(err) => {
                log::error!("there was an watcher error while attempting to bundle: {:?}", err);
            }
        }
    }).expect("failed to create watcher");

    watcher
        .watch(&watch_path, RecursiveMode::Recursive)
        .expect("failed to watch input path");

    std::thread::park();

    Ok(())
}