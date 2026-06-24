use crate::plugin::{self, PluginRegistry};

use super::get_plugin;
use anyhow::Result;

fn flush() {
    for msg in plugin::drain_reports() {
        println!("{msg}");
    }
}

pub(crate) fn install(
    registry: &PluginRegistry,
    language: &str,
    version: Option<&str>,
    no_default: bool,
) -> Result<()> {
    let p = get_plugin(registry, language)?;
    let installed_version = p.install(version)?;
    flush();
    p.use_version(&installed_version, !no_default)?;
    flush();
    p.post_install(&installed_version)?;
    flush();
    Ok(())
}
