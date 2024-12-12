use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use darklua_core::{Options, Resources, WorkerTree};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, DebouncedEvent};

use crate::cli::{error::CliError, process::Options as ProcessOptions, CommandResult};

const FILE_WATCHING_DEBOUNCE_DURATION_MILLIS: u64 = 400;
const DEFAULT_CONFIG_PATHS: [&str; 2] = [".darklua.json", ".darklua.json5"];

enum WatcherSignal {
    Exit,
    Watch(PathBuf),
    Unwatch(PathBuf),
}

pub struct FileWatcher {
    resources: Resources,
    sender: Sender<WatcherSignal>,
    receiver: Option<Receiver<WatcherSignal>>,
    worker_tree: Option<WorkerTree>,
    process_option: ProcessOptions,
    extra_file_watch: HashSet<PathBuf>,
    current_working_path: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new(process_option: &ProcessOptions) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            resources: Resources::from_file_system(),
            sender,
            receiver: Some(receiver),
            worker_tree: None,
            process_option: process_option.clone(),
            extra_file_watch: Default::default(),
            current_working_path: env::current_dir().ok(),
        }
    }

    fn run_worker_tree(&mut self) {
        let options = self.build_options();
        if let Some(worker_tree) = self.worker_tree.as_mut() {
            log_darklua_error(worker_tree.process(&self.resources, options), || ());
        } else {
            self.worker_tree = log_darklua_error(
                darklua_core::process(&self.resources, options).map(Some),
                || None,
            );
        }

        self.update_extra_file_watch();
    }

    fn build_options(&self) -> Options {
        self.process_option.get_process_options()
    }

    pub fn start(mut self) -> CommandResult {
        self.run_worker_tree();
        self.setup_ctrl_exit()?;

        let receiver = self
            .receiver
            .take()
            .expect("file watcher channel receiver should exist");

        let input_path = self.process_option.input_path.clone();
        let config_path = self.process_option.config.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(FILE_WATCHING_DEBOUNCE_DURATION_MILLIS),
            None,
            move |events: DebounceEventResult| match events {
                Ok(events) => {
                    self.process_events(events);
                    self.run_worker_tree();
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

        log::debug!("start watching file system on {}", input_path.display());

        debouncer
            .watch(&input_path, RecursiveMode::Recursive)
            .map_err(|err| {
                log::error!(
                    "unable to start watching file system at `{}`: {}",
                    input_path.display(),
                    err
                );
                CliError::new(1)
            })?;

        if let Some(config) = &config_path {
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
    }

    fn setup_ctrl_exit(&self) -> Result<(), CliError> {
        let sender = self.sender.clone();
        ctrlc::set_handler(move || {
            sender
                .send(WatcherSignal::Exit)
                .expect("unable to send signal to terminate")
        })
        .map_err(|err| {
            log::error!("unable to set Ctrl-C handler: {}", err);
            CliError::new(1)
        })?;
        Ok(())
    }

    fn process_events(&mut self, events: Vec<DebouncedEvent>) {
        if events.is_empty() {
            return;
        }
        let current_path = self.current_working_path.as_ref();

        let worker_tree = if let Some(worker_tree) = self.worker_tree.as_mut() {
            worker_tree
        } else {
            return;
        };

        log::debug!("file watch has detected changes");

        let mut has_created = false;

        for event in events {
            let mut paths_iterator = event.event.paths.iter().map(|path| {
                current_path
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
                        log::trace!("file watcher: {} '{}'", event_display, path.display());
                    }
                }
            }

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
        }

        if has_created {
            self.worker_collect_work();
        }
    }

    fn worker_collect_work(&mut self) {
        let options = self.build_options();
        if let Some(worker_tree) = self.worker_tree.as_mut() {
            log_darklua_error(worker_tree.collect_work(&self.resources, &options), || ());
        }
    }

    fn update_extra_file_watch(&mut self) {
        if let Some(worker_tree) = self.worker_tree.as_ref() {
            let files: HashSet<_> = worker_tree
                .iter_external_dependencies()
                .map(ToOwned::to_owned)
                .collect();

            for last_file in self.extra_file_watch.difference(&files) {
                log::debug!("stop file watching on '{}'", last_file.display());
                if let Err(err) = self
                    .sender
                    .send(WatcherSignal::Unwatch(last_file.to_path_buf()))
                {
                    log::warn!(
                        "unable to send signal to unwatch '{}': {}",
                        last_file.display(),
                        err
                    );
                }
            }

            for new_file in files.difference(&self.extra_file_watch) {
                log::debug!("start file watching on '{}'", new_file.display());
                if let Err(err) = self
                    .sender
                    .send(WatcherSignal::Watch(new_file.to_path_buf()))
                {
                    log::warn!(
                        "unable to send signal to watch '{}': {}",
                        new_file.display(),
                        err
                    );
                }
            }

            self.extra_file_watch = files;
        }
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
