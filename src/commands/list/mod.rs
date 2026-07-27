pub(crate) mod remote;

pub(crate) use remote::list_remote;

use lvm::language::{self, LanguageRegistry};

use crate::commands::{get_language, output};
use anyhow::Result;
use lvm::core::display;

/// Format a version line with optional color highlighting.
fn format_version_line(v: &str, use_color: bool) -> String {
    if use_color {
        if v.contains(language::CURRENT_MARKER) || v.contains(language::CURRENT_DEFAULT_MARKER) {
            format!("{}{}{}", display::COLOR_GREEN_BOLD, v, display::COLOR_RESET)
        } else if v.contains(language::DEFAULT_MARKER) {
            format!("{}{}{}", display::COLOR_YELLOW, v, display::COLOR_RESET)
        } else {
            v.to_string()
        }
    } else {
        v.to_string()
    }
}

/// List locally installed language versions
pub(crate) fn list(registry: &LanguageRegistry, language: &str) -> Result<()> {
    let lang = get_language(registry, language)?;
    let versions = lang.list_installed()?;
    if versions.is_empty() {
        output::info("No versions installed");
    } else {
        let formatted = lang.format_installed(&versions)?;
        let use_color = display::use_color();
        for v in &formatted {
            println!("{}", format_version_line(v, use_color));
        }
    }
    Ok(())
}
