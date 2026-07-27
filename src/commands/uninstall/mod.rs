use lvm::language::LanguageRegistry;

use crate::commands::{flush, get_language};
use anyhow::Result;

pub(crate) fn uninstall(registry: &LanguageRegistry, language: &str, version: &str) -> Result<()> {
    let p = get_language(registry, language)?;
    let result = p.uninstall(version);
    flush();
    result
}
