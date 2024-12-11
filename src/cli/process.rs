use crate::cli::error::CliError;
use crate::cli::utils::maybe_plural;
use crate::cli::{CommandResult, GlobalOptions};

use clap::Args;
use darklua_core::{GeneratorParameters, Resources};
use notify::EventKind;
#[cfg(not(target_arch = "wasm32"))]
use notify::RecursiveMode;
#[cfg(not(target_arch = "wasm32"))]
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::collections::HashSet;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const FILE_WATCHING_DEBOUNCE_DURATION_MILLIS: u64 = 400;

#[derive(Debug, Args, Clone)]
pub struct Options {
    /// Path to the lua file to process.
    input_path: PathBuf,
    /// Where to output the result.
    output_path: PathBuf,
    /// Choose a specific configuration file.
    #[arg(long, short, alias = "config-path")]
    config: Option<PathBuf>,
    /// Choose how Lua code is formatted ('dense', 'readable' or 'retain_lines').
    /// This will override the format given by the configuration file.
    #[arg(long)]
    format: Option<LuaFormat>,
    /// Watch files and directories for changes and automatically re-run
    #[arg(long, short)]
    watch: bool,
}

#[derive(Debug, Copy, Clone)]
enum LuaFormat {
    Dense,
    Readable,
    RetainLines,
}

impl FromStr for LuaFormat {
    type Err = String;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format {
            "dense" => Ok(Self::Dense),
            "readable" => Ok(Self::Readable),
            // keep "retain-lines" for back-compatibility
            "retain_lines" | "retain-lines" => Ok(Self::RetainLines),
            _ => Err(format!(
                "format '{}' does not exist! (possible options are: 'dense', 'readable' or 'retain_lines'",
                format
            )),
        }
    }
}

fn process(resources: Resources, process_options: darklua_core::Options) -> CommandResult {
    let process_start_time = Instant::now();

    let result = darklua_core::process(&resources, process_options).map_err(|err| {
        log::error!("{}", err);
        CliError::new(1)
    })?;

    let process_duration = durationfmt::to_string(process_start_time.elapsed());

    let success_count = result.success_count();

    println!(
        "successfully processed {} file{} (in {})",
        success_count,
        maybe_plural(success_count),
        process_duration
    );

    let errors = result.collect_errors();

    if errors.is_empty() {
        Ok(())
    } else {
        let error_count = errors.len();
        eprintln!(
            "{}{} error{} happened:",
            if success_count > 0 { "but " } else { "" },
            error_count,
            maybe_plural(error_count)
        );

        for error in errors {
            eprintln!("-> {}", error);
        }

        Err(CliError::new(1))
    }
}

impl Options {
    fn get_process_options(&self) -> darklua_core::Options {
        let mut process_options =
            darklua_core::Options::new(&self.input_path).with_output(&self.output_path);

        if let Some(config) = self.config.as_ref() {
            process_options = process_options.with_configuration_at(config);
        }

        if let Some(format) = self.format {
            process_options = process_options.with_generator_override(match format {
                LuaFormat::Dense => GeneratorParameters::default_dense(),
                LuaFormat::Readable => GeneratorParameters::default_readable(),
                LuaFormat::RetainLines => GeneratorParameters::RetainLines,
            })
        }
        process_options
    }
}

const DEFAULT_CONFIG_PATHS: [&str; 2] = [".darklua.json", ".darklua.json5"];

pub fn run(options: &Options, _global: &GlobalOptions) -> CommandResult {
    log::debug!("running `process`: {:?}", options);

    if cfg!(not(target_arch = "wasm32")) && options.watch {
        let watcher_options = options.clone();

        let resources = Resources::from_file_system();

        let mut worker_tree = setup_worker_tree(&resources, &watcher_options);

        let (sender, receiver) = mpsc::channel();
        let process_sender = sender.clone();

        let current_path = current_dir().ok();

        let mut extra_file_watch = HashSet::new();

        let mut debouncer = new_debouncer(
            Duration::from_millis(FILE_WATCHING_DEBOUNCE_DURATION_MILLIS),
            None,
            move |events: DebounceEventResult| match events {
                Ok(events) => {
                    for event in events.iter() {
                        if let Some(worker_tree) = worker_tree.as_mut() {
                            log::debug!("file watch has detected changes");

                            let mut paths_iterator = event.event.paths.iter().map(|path| {
                                current_path
                                    .as_ref()
                                    .and_then(|current_path| path.strip_prefix(current_path).ok())
                                    .unwrap_or(path)
                            });

                            if log::log_enabled!(log::Level::Trace) {
                                let event_display = match event.kind {
                                    EventKind::Any => Some("unknown operation on"),
                                    EventKind::Create(_) => Some("created"),
                                    EventKind::Modify(_) => Some("modified"),
                                    EventKind::Remove(_) => Some("removed"),
                                    EventKind::Access(_) | EventKind::Other => None,
                                };
                                if let Some(event_display) = event_display {
                                    for path in event.event.paths.iter() {
                                        log::trace!(
                                            "file watcher: {} '{}'",
                                            event_display,
                                            path.display()
                                        );
                                    }
                                }
                            }

                            let mut has_created = false;

                            match event.kind {
                                EventKind::Any => {
                                    for path in paths_iterator {
                                        has_created = true;
                                        worker_tree.source_changed(path);
                                    }
                                }
                                EventKind::Create(_create_kind) => {
                                    has_created = paths_iterator.next().is_some();
                                }
                                EventKind::Modify(_modify_kind) => {
                                    for path in paths_iterator {
                                        worker_tree.source_changed(path);
                                    }
                                }
                                EventKind::Remove(_remove_kind) => {
                                    for path in paths_iterator {
                                        worker_tree.remove_source(path);
                                    }
                                }
                                EventKind::Access(_) | EventKind::Other => {}
                            }

                            if has_created {
                                log_darklua_error(
                                    worker_tree.collect_work(
                                        &resources,
                                        &watcher_options.get_process_options(),
                                    ),
                                    || (),
                                );
                            }
                        }
                    }

                    if let Some(worker_tree) = worker_tree.as_mut() {
                        log_darklua_error(
                            worker_tree.process(&resources, watcher_options.get_process_options()),
                            || (),
                        );
                    } else {
                        worker_tree = setup_worker_tree(&resources, &watcher_options);
                    }

                    if let Some(worker_tree) = worker_tree.as_ref() {
                        let files: HashSet<_> = worker_tree
                            .iter_external_dependencies()
                            .map(ToOwned::to_owned)
                            .collect();

                        for last_file in extra_file_watch.difference(&files) {
                            log::debug!("stop file watching on '{}'", last_file.display());
                            if let Err(err) =
                                process_sender.send(WatcherSignal::Unwatch(last_file.to_path_buf()))
                            {
                                log::warn!(
                                    "unable to send signal to unwatch '{}': {}",
                                    last_file.display(),
                                    err
                                );
                            }
                        }

                        for new_file in files.difference(&extra_file_watch) {
                            log::debug!("start file watching on '{}'", new_file.display());
                            if let Err(err) =
                                process_sender.send(WatcherSignal::Watch(new_file.to_path_buf()))
                            {
                                log::warn!(
                                    "unable to send signal to watch '{}': {}",
                                    new_file.display(),
                                    err
                                );
                            }
                        }

                        extra_file_watch = files;
                    }
                }
                Err(errors) => {
                    for err in errors {
                        log::error!(
                            "an error occured while watching file system for changes: {}",
                            err
                        );
                    }
                }
            },
        )
        .map_err(|err| {
            log::error!("unable to create file watcher: {}", err);
            CliError::new(1)
        })?;

        log::debug!(
            "start watching file system {}",
            options.input_path.display()
        );

        debouncer
            .watch(&options.input_path, RecursiveMode::Recursive)
            .map_err(|err| {
                log::error!(
                    "unable to start watching file system at `{}`: {}",
                    options.input_path.display(),
                    err
                );
                CliError::new(1)
            })?;

        if let Some(config) = options.config.as_ref() {
            log::debug!("start watching provided config path {}", config.display());
            debouncer
                .watch(config, RecursiveMode::NonRecursive)
                .map_err(|err| {
                    log::error!(
                        "unable to start watching file system at `{}`: {}",
                        config.display(),
                        err
                    );
                    CliError::new(1)
                })?;
        } else {
            for path in DEFAULT_CONFIG_PATHS.iter().map(Path::new) {
                if path.exists() {
                    log::debug!("start watching default config path {}", path.display());
                    debouncer
                        .watch(path, RecursiveMode::NonRecursive)
                        .map_err(|err| {
                            log::error!(
                                "unable to start watching file system at `{}`: {}",
                                path.display(),
                                err
                            );
                            CliError::new(1)
                        })?;
                }
            }
        }

        ctrlc::set_handler(move || {
            sender
                .send(WatcherSignal::Exit)
                .expect("unable to send signal to terminate")
        })
        .map_err(|err| {
            log::error!("unable to set Ctrl-C handler: {}", err);
            CliError::new(1)
        })?;

        log::debug!("waiting for Ctrl-C to close the program");

        loop {
            match receiver.recv().expect("Could not receive from channel.") {
                WatcherSignal::Exit => break,
                WatcherSignal::Watch(path) => {
                    match debouncer.watch(&path, RecursiveMode::NonRecursive) {
                        Ok(()) => {}
                        Err(err) => {
                            log::error!(
                                "unable to start watching file system at `{}`: {}",
                                path.display(),
                                err
                            );
                        }
                    }
                }
                WatcherSignal::Unwatch(path) => match debouncer.unwatch(&path) {
                    Ok(()) => {}
                    Err(err) => {
                        log::error!(
                            "unable to stop watching file system at `{}`: {}",
                            path.display(),
                            err
                        );
                    }
                },
            }
        }

        Ok(())
    } else {
        let resources = Resources::from_file_system();

        process(resources, options.get_process_options())
    }
}

fn log_darklua_error<T>(
    result: Result<T, darklua_core::DarkluaError>,
    else_result: impl Fn() -> T,
) -> T {
    result
        .inspect_err(|err| {
            log::error!("{}", err);
        })
        .unwrap_or_else(|_| else_result())
}

fn setup_worker_tree(
    resources: &Resources,
    watcher_options: &Options,
) -> Option<darklua_core::WorkerTree> {
    log_darklua_error(
        darklua_core::process(resources, watcher_options.get_process_options()).map(Some),
        || None,
    )
}

enum WatcherSignal {
    Exit,
    Watch(PathBuf),
    Unwatch(PathBuf),
}
