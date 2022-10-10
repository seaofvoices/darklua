mod configuration;
mod error;
mod options;
mod resources;
mod utils;
mod work_item;
mod worker;

use std::path::Path;

pub use resources::Resources;
use work_item::WorkItem;

pub use error::{DarkluaError, DarkluaResult};
pub use options::Options;
use worker::Worker;

use self::utils::normalize_path;

pub fn process(resources: &Resources, options: Options) -> Result<(), Vec<DarkluaError>> {
    let worker = Worker::new(resources);

    if let Some(output) = options.output().map(Path::to_path_buf) {
        if resources.is_file(options.input()).map_err(element_to_vec)? {
            if resources.is_directory(&output).map_err(element_to_vec)? {
                let file_name = options
                    .input()
                    .file_name()
                    .ok_or_else(|| {
                        DarkluaError::custom(format!(
                            "unable to extract file name from `{}`",
                            options.input().display()
                        ))
                    })
                    .map_err(element_to_vec)?;

                worker.process(
                    once_ok(WorkItem::new(options.input(), output.join(file_name))),
                    options,
                )
            } else if resources.is_file(&output).map_err(element_to_vec)?
                || output.extension().is_some()
            {
                worker.process(once_ok(WorkItem::new(options.input(), output)), options)
            } else {
                let file_name = options
                    .input()
                    .file_name()
                    .ok_or_else(|| {
                        DarkluaError::custom(format!(
                            "unable to extract file name from `{}`",
                            options.input().display()
                        ))
                    })
                    .map_err(element_to_vec)?;

                worker.process(
                    once_ok(WorkItem::new(options.input(), output.join(file_name))),
                    options,
                )
            }
        } else {
            let input = options.input().to_path_buf();

            worker.process(
                resources.collect_work(&input).map(|source| {
                    let source = normalize_path(&source);
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
        worker.process(
            resources
                .collect_work(options.input().to_path_buf())
                .map(|source| Ok(WorkItem::new_in_place(source))),
            options,
        )
    }
}

#[inline]
fn element_to_vec<T>(element: impl Into<T>) -> Vec<T> {
    vec![element.into()]
}

#[inline]
fn once_ok<T, E>(value: T) -> impl Iterator<Item = Result<T, E>> {
    std::iter::once(Ok(value))
}
