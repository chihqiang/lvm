pub(crate) mod remote;

pub(crate) use remote::list_remote;

use crate::language::{self, LanguageRegistry};

use crate::commands::{get_language, output};
use crate::config;
use anyhow::Result;

/// 列出本地已安装的语言版本
pub(crate) fn list(registry: &LanguageRegistry, language: &str) -> Result<()> {
    let lang = get_language(registry, language)?;
    let versions = lang.list_installed()?;
    if versions.is_empty() {
        output::info("No versions installed");
    } else {
        let formatted = lang.format_installed(&versions)?;
        let use_color = config::use_color();
        for v in &formatted {
            if use_color {
                if v.contains(language::CURRENT_MARKER)
                    || v.contains(language::CURRENT_DEFAULT_MARKER)
                {
                    println!(
                        "{}{}{}",
                        config::color_green_bold(),
                        v,
                        config::color_reset()
                    );
                } else if v.contains(language::DEFAULT_MARKER) {
                    println!("{}{}{}", config::color_yellow(), v, config::color_reset());
                } else {
                    println!("{v}");
                }
            } else {
                println!("{v}");
            }
        }
    }
    Ok(())
}
