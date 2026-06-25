pub(crate) mod alias;
pub(crate) mod cache;
pub(crate) mod cli;
pub(crate) mod current;
pub(crate) mod debug;
pub(crate) mod dispatch;
pub(crate) mod env;
pub(crate) mod hook;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod output;
pub(crate) mod prune;
pub(crate) mod reinstall;
pub(crate) mod uninstall;
pub(crate) mod use_version;
pub(crate) mod which;

use crate::config;

pub(crate) use alias::{alias, unalias};
pub(crate) use cache::cache_clear;
pub(crate) use current::{current, current_all};
pub(crate) use debug::debug;

pub(crate) use env::{env, env_completions};
pub(crate) use hook::hook;
pub(crate) use install::install;
pub(crate) use list::{list, list_remote};
pub(crate) use prune::prune;
pub(crate) use reinstall::reinstall_packages;
pub(crate) use uninstall::uninstall;
pub(crate) use use_version::use_version;
pub(crate) use which::which;

use crate::language::{self, Language, LanguageRegistry};
use anyhow::{Context, Result};

pub(crate) fn flush() {
    for msg in language::drain_reports() {
        println!("{msg}");
    }
}

pub(crate) fn get_language<'a>(
    registry: &'a LanguageRegistry,
    name: &str,
) -> Result<&'a dyn Language> {
    registry.get(name).with_context(|| {
        let available = registry.list_names().join(config::list_separator());
        format!("Unknown language '{name}', available: {available}")
    })
}

pub(crate) fn binary_dir(
    registry: &LanguageRegistry,
    language: &str,
    version: &str,
) -> Result<std::path::PathBuf> {
    let lang = get_language(registry, language)?;
    let bin_path = lang.binary_path(version)?;
    std::path::Path::new(&bin_path)
        .parent()
        .map(std::path::Path::to_path_buf)
        .context("Cannot determine binary directory")
}
