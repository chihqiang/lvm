use anyhow::{Result, bail};
use semver::VersionReq;

use crate::core::lvmrc;
use crate::language::LanguageRegistry;

pub fn is_version_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if VersionReq::parse(s).is_ok() {
        return true;
    }
    let stripped = s.trim_start_matches('v');
    // Every dot-separated segment must be a non-empty run of digits, which
    // rejects malformed inputs like "22..1" or "22a".
    !stripped.is_empty()
        && stripped
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

pub fn resolve_install_args(
    arg_lang: Option<&str>,
    arg_ver: Option<&str>,
    registry: &LanguageRegistry,
) -> Result<Vec<(String, Option<String>)>> {
    match (arg_lang, arg_ver) {
        (Some(lang), None) if is_version_like(lang) => {
            let names = registry.list_names();
            if names.len() == 1 {
                Ok(vec![(names[0].to_string(), Some(lang.to_string()))])
            } else if names.is_empty() {
                bail!("No languages registered, cannot resolve version: {lang}")
            } else {
                let available = names.join(", ");
                bail!(
                    "Ambiguous argument '{lang}': looks like a version, but multiple languages ({available}) are registered. Use 'lvm install <language> {lang}'"
                )
            }
        }
        (Some(lang), None) => Ok(vec![(lang.to_string(), None)]),
        (Some(lang), Some(ver)) => Ok(vec![(lang.to_string(), Some(ver.to_string()))]),
        (None, _) => {
            if let Some(path) = lvmrc::find_lvmrc() {
                let map = lvmrc::parse_lvmrc(&path)?;
                let vec: Vec<_> = map.into_iter().map(|(k, v)| (k, Some(v))).collect();
                if vec.is_empty() {
                    bail!(".lvmrc exists but contains no language-version mappings");
                }
                return Ok(vec);
            }
            for name in registry.list_names() {
                if let Some(lang) = registry.get(name)
                    && let Some(v) = lang.rc_version()?
                {
                    return Ok(vec![(name.to_string(), Some(v))]);
                }
            }
            bail!("No .lvmrc or .nvmrc found. Create one or specify arguments")
        }
    }
}

pub fn write_current_versions_to_lvmrc(
    registry: &LanguageRegistry,
    plans: &[(String, Option<String>)],
) -> Result<Option<String>> {
    let mut count = 0;
    for (lang, _) in plans {
        if let Some(language) = registry.get(lang)
            && let Some(cur) = language.current_version()?
        {
            lvmrc::write_lvmrc(lang, &cur)?;
            count += 1;
        }
    }
    if count > 0 {
        Ok(Some(format!("Wrote {count} language(s) to .lvmrc")))
    } else {
        Ok(None)
    }
}
