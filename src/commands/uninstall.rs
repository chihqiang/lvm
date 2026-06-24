use crate::plugin::{self, PluginRegistry};

use super::get_plugin;
use anyhow::Result;

pub(crate) fn uninstall(registry: &PluginRegistry, language: &str, version: &str) -> Result<()> {
    let p = get_plugin(registry, language)?;
    let result = p.uninstall(version);
    for msg in plugin::drain_reports() {
        println!("{msg}");
    }
    result
}
