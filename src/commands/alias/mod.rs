pub(crate) mod unalias;

pub(crate) use unalias::unalias;

use lvm::core::alias;

use crate::commands::output;
use anyhow::{Result, bail};

/// 管理版本别名（查看/设置）
pub(crate) fn alias(language: &str, name: Option<&str>, version: Option<&str>) -> Result<()> {
    match (name, version) {
        (None, None) => {
            let names = alias::list_alias_names(language)?;
            if names.is_empty() {
                output::info(format!("No aliases for {language}"));
            } else {
                for n in &names {
                    if let Some(val) = alias::get_alias(language, n)? {
                        output::info(format!("{n} -> {val}"));
                    }
                }
            }
        }
        (Some(name), None) => {
            if let Some(val) = alias::get_alias(language, name)? {
                output::info(format!("{name} -> {val}"));
            } else {
                bail!("Alias '{name}' not found");
            }
        }
        (Some(name), Some(version)) => {
            alias::set_alias(language, name, version)?;
            output::info(format!("{name} -> {version}"));
        }
        (None, Some(_)) => bail!("Alias name is required"),
    }
    Ok(())
}
