use lvm::core::alias;

use crate::commands::output;
use anyhow::Result;

/// Remove a version alias
pub(crate) fn unalias(language: &str, name: &str) -> Result<()> {
    alias::remove_alias(language, name)?;
    output::info(format!("Removed alias '{name}' for {language}"));
    Ok(())
}
