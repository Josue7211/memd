use super::*;

#[path = "obsidian_import_runtime.rs"]
mod obsidian_import_runtime;

pub(crate) use obsidian_import_runtime::run_obsidian_import;

#[path = "obsidian_compile_runtime.rs"]
mod obsidian_compile_runtime;

pub(crate) use obsidian_compile_runtime::run_obsidian_compile;

#[path = "obsidian_watch_runtime.rs"]
mod obsidian_watch_runtime;

pub(crate) use obsidian_watch_runtime::run_obsidian_watch;

#[path = "obsidian_open_runtime.rs"]
mod obsidian_open_runtime;

pub(crate) use obsidian_open_runtime::run_obsidian_open;

#[path = "obsidian_status_runtime.rs"]
mod obsidian_status_runtime;

pub(crate) use obsidian_status_runtime::run_obsidian_status;
