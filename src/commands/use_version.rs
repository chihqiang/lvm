use crate::config;
use crate::plugin::{self, PluginRegistry};

use super::get_plugin;
use super::output;
use anyhow::Result;

fn flush() {
    for msg in plugin::drain_reports() {
        println!("{msg}");
    }
}

pub(crate) fn use_version(
    registry: &PluginRegistry,
    language: &str,
    version: Option<&str>,
    set_default: bool,
) -> Result<()> {
    let p = get_plugin(registry, language)?;

    let version = match version {
        Some(v) => v.to_string(),
        None => {
            if let Some(v) = config::lvmrc::read_lvmrc_version(language)? {
                v
            } else if let Some(v) = config::alias::get_default_version(language)? {
                v
            } else {
                let latest = p.latest_version()?;
                output::info(format!("Using latest {language} version {latest}"));
                latest
            }
        }
    };

    if version == config::display::system_keyword() {
        let pname = p.name();
        let link = config::lvm_home()
            .join(config::current_dir_name())
            .join(pname);
        let _ = plugin::remove_symlink(&link);
        let _ =
            plugin::remove_symlink(&config::lvm_home().join(config::bin_dir_name()).join(pname));
        output::info(format!("Using system {language}"));
        return Ok(());
    }

    let installed = p.install(Some(&version))?;
    flush();
    p.use_version(&installed, set_default)?;
    flush();
    Ok(())
}
