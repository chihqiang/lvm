//! LVM - Language Version Manager
//! Multi-language version manager with pluggable architecture

#![allow(clippy::multiple_crate_versions)]

mod commands;

use anyhow::Result;
use lvm::language;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut registry = language::LanguageRegistry::new();
    registry.register_all();

    let mut cmd = commands::cli::build_cli();
    let cli = cmd.get_matches_mut();

    commands::dispatch::execute(&mut cmd, &cli, &registry)
}
