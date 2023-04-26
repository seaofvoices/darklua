mod configuration;
mod error;
mod options;
mod process_result;
mod resources;
mod utils;
mod work_cache;
mod work_item;
mod worker;

use std::path::Path;

pub use configuration::{Configuration, GeneratorParameters};
pub use error::{DarkluaError, DarkluaResult};
pub use options::Options;
pub use process_result::ProcessResult;
pub use resources::Resources;
use work_item::WorkItem;
use worker::Worker;

use crate::utils::normalize_path;

pub fn process(resources: &Resources, options: Options) -> ProcessResult {
    match private_process(resources, options) {
        Ok(result) | Err(result) => result,
    }
}

fn private_process(
    resources: &Resources,
    options: Options,
) -> Result<ProcessResult, ProcessResult> {
    let worker = Worker::new(resources);

    if let Some(output) = options.output().map(Path::to_path_buf) {
        if resources.is_file(options.input())? {
            if resources.is_directory(&output)? {
                let file_name = options.input().file_name().ok_or_else(|| {
                    DarkluaError::custom(format!(
                        "unable to extract file name from `{}`",
                        options.input().display()
                    ))
                })?;

                worker.process(
                    once_ok(WorkItem::new(options.input(), output.join(file_name))),
                    options,
                )
            } else if resources.is_file(&output)? || output.extension().is_some() {
                worker.process(once_ok(WorkItem::new(options.input(), output)), options)
            } else {
                let file_name = options.input().file_name().ok_or_else(|| {
                    DarkluaError::custom(format!(
                        "unable to extract file name from `{}`",
                        options.input().display()
                    ))
                })?;

                worker.process(
                    once_ok(WorkItem::new(options.input(), output.join(file_name))),
                    options,
                )
            }
        } else {
            let input = options.input().to_path_buf();

            worker.process(
                resources.collect_work(&input).map(|source| {
                    let source = normalize_path(source);
                    source
                        .strip_prefix(&input)
                        .map(|relative| WorkItem::new(&source, output.join(relative)))
                        .map_err(|err| {
                            DarkluaError::custom(format!(
                                "unable to remove path prefix `{}` from `{}`: {}",
                                input.display(),
                                source.display(),
                                err
                            ))
                        })
                }),
                options,
            )
        }
    } else {
        let input = options.input().to_path_buf();
        worker.process(
            resources
                .collect_work(input)
                .map(|source| Ok(WorkItem::new_in_place(source))),
            options,
        )
    }
}

#[inline]
fn once_ok<T, E>(value: T) -> impl Iterator<Item = Result<T, E>> {
    std::iter::once(Ok(value))
}
