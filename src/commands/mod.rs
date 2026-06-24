pub mod alias;
pub mod cache;
pub mod cli;
pub mod current;
pub mod debug;
pub mod env;
pub mod hook;
pub mod install;
pub mod list;
pub mod list_remote;
pub mod output;
pub mod reinstall;
pub mod unalias;
pub mod uninstall;
pub mod use_version;
pub mod which;

use crate::config;

pub(crate) use alias::alias;
pub(crate) use cache::cache_clear;
pub(crate) use current::{current, current_all};
pub(crate) use debug::debug;
pub(crate) use env::env;
pub(crate) use env::env_completions;
pub(crate) use hook::hook;
pub(crate) use install::install;
pub(crate) use list::list;
pub(crate) use list_remote::list_remote;
pub(crate) use reinstall::reinstall_packages;
pub(crate) use unalias::unalias;
pub(crate) use uninstall::uninstall;
pub(crate) use use_version::use_version;
pub(crate) use which::which;

use crate::plugin::{Plugin, PluginRegistry};
use anyhow::{Context, Result};

pub(crate) fn get_plugin<'a>(registry: &'a PluginRegistry, name: &str) -> Result<&'a dyn Plugin> {
    registry.get(name).with_context(|| {
        let available = registry.list_names().join(config::list_separator());
        format!("Unknown language '{name}', available: {available}")
    })
}

/// 返回指定语言版本的可执行文件所在目录
pub(crate) fn binary_dir(
    registry: &PluginRegistry,
    language: &str,
    version: &str,
) -> Result<std::path::PathBuf> {
    let plugin = get_plugin(registry, language)?;
    let bin_path = plugin.binary_path(version)?;
    std::path::Path::new(&bin_path)
        .parent()
        .map(std::path::Path::to_path_buf)
        .context("Cannot determine binary directory")
}
