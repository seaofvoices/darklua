mod configuration;
mod error;
mod options;
mod resources;
mod utils;
mod work_item;
mod worker;

use std::iter;

pub use resources::Resources;
use work_item::WorkItem;

pub use error::DarkluaError;
pub use options::Options;
use worker::Worker;

use self::utils::normalize_path;

pub fn process(resources: &Resources, options: &Options) -> Result<(), Vec<DarkluaError>> {
    let worker = Worker::new(resources);

    if let Some(output) = options.output() {
        if resources.is_file(options.input()).map_err(element_to_vec)? {
            if resources.is_directory(output).map_err(element_to_vec)? {
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
                    iter::once(WorkItem::new(options.input(), output.join(file_name))),
                    options,
                )
            } else if resources.is_file(output).map_err(element_to_vec)?
                || output.extension().is_some()
            {
                worker.process(iter::once(WorkItem::new(options.input(), output)), options)
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
                    iter::once(WorkItem::new(options.input(), output.join(file_name))),
                    options,
                )
            }
        } else {
            let base = options.input();

            worker.process(
                resources.collect_work(options.input()).map(|source| {
                    let source = normalize_path(&source);
                    let relative = source.strip_prefix(base).expect("todo: this should work");
                    let work_output = output.join(relative);
                    WorkItem::new(source, work_output)
                }),
                options,
            )
        }
    } else {
        worker.process(
            resources
                .collect_work(options.input())
                .map(|source| WorkItem::new_in_place(source)),
            options,
        )
    }
}

#[inline]
fn element_to_vec<T>(element: impl Into<T>) -> Vec<T> {
    vec![element.into()]
}
