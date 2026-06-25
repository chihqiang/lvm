use crate::config;

use crate::commands::output;
use anyhow::Result;

/// 删除指定版本别名
pub(crate) fn unalias(language: &str, name: &str) -> Result<()> {
    config::remove_alias(language, name)?;
    output::info(format!("Removed alias '{name}' for {language}"));
    Ok(())
}
