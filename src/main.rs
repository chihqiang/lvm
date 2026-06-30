//! LVM - Language Version Manager
//! 多语言版本管理工具，支持通过插件式架构扩展新的语言

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
    registry.register(Box::new(language::node::NodeLanguage));
    registry.register(Box::new(language::go::GoLanguage));
    registry.register(Box::new(language::java::JavaLanguage));
    registry.register(Box::new(language::python::PythonLanguage));
    registry.register(Box::new(language::dart::DartLanguage));
    registry.register(Box::new(language::flutter::FlutterLanguage));
    registry.register(Box::new(language::kotlin::KotlinLanguage));
    registry.register(Box::new(language::rust::RustLanguage));

    let mut cmd = commands::cli::build_cli();
    let cli = cmd.get_matches_mut();

    commands::dispatch::execute(&mut cmd, &cli, &registry)
}
