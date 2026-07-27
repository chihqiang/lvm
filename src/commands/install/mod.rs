use lvm::language::{self, LanguageRegistry};

use crate::commands::{flush, get_language};
use anyhow::{Result, anyhow};

pub(crate) fn install(
    registry: &LanguageRegistry,
    language: &str,
    version: Option<&str>,
    no_default: bool,
) -> Result<()> {
    let p = get_language(registry, language)?;
    let installed_version = p.install(version)?;
    flush();
    p.use_version(&installed_version, !no_default)?;
    flush();
    p.post_install(&installed_version)?;
    flush();
    Ok(())
}

/// Install multiple language versions. Runs in parallel when more than one plan
/// is provided (e.g. from `.lvmrc`).
pub(crate) fn install_plans(
    registry: &LanguageRegistry,
    plans: &[(String, Option<String>)],
    no_default: bool,
) -> Result<()> {
    if plans.len() <= 1 {
        for (lang, ver) in plans {
            install(registry, lang, ver.as_deref(), no_default)?;
        }
        return Ok(());
    }

    language::set_parallel_downloads(true);
    let result = run_parallel_installs(registry, plans, no_default);
    language::set_parallel_downloads(false);
    result
}

fn run_parallel_installs(
    registry: &LanguageRegistry,
    plans: &[(String, Option<String>)],
    no_default: bool,
) -> Result<()> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = plans
            .iter()
            .map(|(lang, ver)| {
                let lang = lang.clone();
                let ver = ver.clone();
                scope.spawn(move || install(registry, &lang, ver.as_deref(), no_default))
            })
            .collect();

        let mut errors = Vec::new();
        for handle in handles {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => errors.push(e),
                Err(_) => errors.push(anyhow!("Install thread panicked")),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.pop().unwrap())
        } else {
            let msg = errors
                .into_iter()
                .map(|e| format!("  - {e:#}"))
                .collect::<Vec<_>>()
                .join("\n");
            Err(anyhow!("Multiple install failures:\n{msg}"))
        }
    })
}
