use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::core::report::report;
use super::version;

/// LTS info is refreshed from the network at most once per day.
const LTS_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const LTS_CACHE_FILENAME: &str = "node-lts.json";

#[derive(Serialize, Deserialize)]
pub(crate) struct LtsInfo {
    pub(crate) latest: Option<String>,
    pub(crate) name_to_ver: HashMap<String, String>,
    pub(crate) ordered: Vec<(String, Option<String>)>,
}

/// Column indices in Node's index.tab format.
/// Format: version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts
const COL_VERSION: usize = 0;
const COL_LTS: usize = 9;
const MIN_COLUMNS: usize = 10;

pub(crate) fn parse_lts_info(text: &str) -> Vec<(String, Option<String>)> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < MIN_COLUMNS {
                return None;
            }
            let version = parts[COL_VERSION].strip_prefix('v')?;
            let lts = parts.get(COL_LTS).and_then(|s| {
                let s = s.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            });
            Some((version.to_string(), lts))
        })
        .collect()
}

/// Thread-safe process-level cache for parsed LTS info.
/// Avoids redundant network fetches and re-parsing within the same process.
static LTS_INFO_CACHE: OnceLock<LtsInfo> = OnceLock::new();

pub(crate) fn get_lts_info() -> Result<&'static LtsInfo> {
    if let Some(info) = LTS_INFO_CACHE.get() {
        return Ok(info);
    }

    let (cached, fresh) = load_lts_cache()?;

    let info = if cached.is_some() && fresh {
        cached.expect("cached is Some")
    } else {
        match version::fetch_index_tab() {
            Ok(text) => {
                let info = build_lts_info(&text);
                // Persist parsed LTS info so hook-driven cd doesn't need the
                // network again for up to LTS_CACHE_TTL.
                if let Err(e) = save_lts_cache(&info) {
                    report(format!("Warning: failed to write LTS cache: {e:#}"));
                }
                info
            }
            // Network failed: fall back to stale cache rather than erroring out.
            Err(e) if cached.is_some() => {
                report(format!(
                    "Warning: could not refresh LTS info ({e:#}), using cached data"
                ));
                cached.expect("cached is Some")
            }
            Err(e) => return Err(e),
        }
    };

    // Only the first thread's result is stored; subsequent calls return the cached value
    let _ = LTS_INFO_CACHE.set(info);
    Ok(LTS_INFO_CACHE.get().expect("LTS info cache was just set"))
}

/// Build an [`LtsInfo`] from the raw Node `index.tab` text.
fn build_lts_info(text: &str) -> LtsInfo {
    let mut ordered = parse_lts_info(text);
    // Normalize to newest-first so the logic below is independent of the
    // upstream ordering of index.tab.
    ordered.sort_by(|a, b| crate::core::version::compare_versions(&b.0, &a.0));

    let mut name_to_ver: HashMap<String, String> = HashMap::new();
    for (ver, lts) in &ordered {
        if let Some(codename) = lts {
            // First occurrence (newest) wins for a given codename.
            name_to_ver
                .entry(codename.to_lowercase())
                .or_insert_with(|| ver.clone());
        }
    }

    let latest = ordered
        .iter()
        .find(|(_, lts)| lts.is_some())
        .map(|(v, _)| v.clone());

    LtsInfo {
        latest,
        name_to_ver,
        ordered,
    }
}

fn lts_cache_file() -> std::path::PathBuf {
    crate::config::cache_path(LTS_CACHE_FILENAME)
}

/// Load cached LTS info. Returns `(info, fresh)` where `fresh` is true only if
/// the cache exists, is readable, and is younger than [`LTS_CACHE_TTL`].
fn load_lts_cache() -> Result<(Option<LtsInfo>, bool)> {
    let path = lts_cache_file();
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((None, false)),
        Err(e) => return Err(e).context("Failed to stat LTS cache"),
    };
    let modified = meta.modified().context("Failed to read LTS cache mtime")?;
    let fresh = modified
        .elapsed()
        .map_or(false, |elapsed| elapsed < LTS_CACHE_TTL);
    match fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<LtsInfo>(&text).ok())
    {
        Some(info) => Ok((Some(info), fresh)),
        None => Ok((None, fresh)),
    }
}

/// Persist LTS info to disk (best-effort; callers may ignore the error).
fn save_lts_cache(info: &LtsInfo) -> Result<()> {
    let path = lts_cache_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }
    let json = serde_json::to_string(info).context("Failed to serialize LTS cache")?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &json).context("Failed to write LTS cache")?;
    fs::rename(&tmp, &path).context("Failed to finalize LTS cache")?;
    Ok(())
}

pub(crate) fn resolve_lts(desc: &str) -> Result<String> {
    let info = get_lts_info()?;

    if desc == "*" || desc.is_empty() {
        return info
            .latest
            .clone()
            .ok_or_else(|| anyhow!("No LTS version found"));
    }

    if let Some(offset_str) = desc.strip_prefix('-')
        && let Ok(n) = offset_str.parse::<usize>()
    {
        // `ordered` is newest-first, so iterating forward gives the LTS
        // releases from newest to oldest.
        let mut lts_versions: Vec<&str> = info
            .ordered
            .iter()
            .filter(|(_, lts)| lts.is_some())
            .map(|(v, _)| v.as_str())
            .collect();
        let mut seen = std::collections::HashSet::new();
        lts_versions.retain(|v| {
            let major = v.split('.').next().unwrap_or_default();
            seen.insert(major.to_string())
        });
        if n < lts_versions.len() {
            return Ok(lts_versions[n].to_string());
        }
        bail!("LTS offset {n} is out of range");
    }

    let lower = desc.to_lowercase();
    if let Some(ver) = info.name_to_ver.get(&lower) {
        return Ok(ver.clone());
    }

    bail!("Unknown LTS release: {desc}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample index.tab lines (version, date, files, npm, v8, uv, zlib,
    /// openssl, modules, lts).
    const SAMPLE_TAB: &str = concat!(
        "version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\n",
        "v23.0.0\t2024-10-15\t17\t10.9.0\t12.4.254.20\t1.48.0\t1.3.1\t3.0.13\t127\t\n",
        "v22.11.0\t2024-10-15\t22\t10.9.0\t12.4.254.20\t1.48.0\t1.3.1\t3.0.13\t127\tJod\n",
        "v20.18.1\t2024-10-15\t21\t10.8.2\t11.3.244.8\t1.48.0\t1.3.1\t3.0.13\t115\tIron\n",
        "v20.17.0\t2024-09-26\t21\t10.8.2\t11.3.244.8\t1.48.0\t1.3.1\t3.0.13\t115\tIron\n",
    );

    #[test]
    fn build_lts_info_parses_mapping_and_latest() {
        let info = build_lts_info(SAMPLE_TAB);

        assert_eq!(info.latest.as_deref(), Some("22.11.0"));
        assert_eq!(info.name_to_ver.get("iron"), Some(&"20.18.1".to_string()));
        assert_eq!(info.name_to_ver.get("jod"), Some(&"22.11.0".to_string()));
        // Latest stable (non-LTS) version is present in ordered, not in LTS map.
        assert!(info.name_to_ver.get("23.0.0").is_none());
    }

    #[test]
    fn lts_info_serde_roundtrip_preserves_data() {
        let info = build_lts_info(SAMPLE_TAB);
        let json = serde_json::to_string(&info).expect("serialize");
        let decoded: LtsInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(decoded.latest, info.latest);
        assert_eq!(decoded.name_to_ver, info.name_to_ver);
        assert_eq!(decoded.ordered, info.ordered);
    }

    #[test]
    fn resolve_lts_by_name_and_offset() {
        let info = build_lts_info(SAMPLE_TAB);
        // Verify name lookup logic used by resolve_lts (newest patch wins).
        assert_eq!(
            info.name_to_ver.get("iron").map(String::as_str),
            Some("20.18.1")
        );
        // `ordered` is newest-first; newest LTS major (22) is first.
        let lts_majors: Vec<&str> = info
            .ordered
            .iter()
            .filter(|(_, lts)| lts.is_some())
            .map(|(v, _)| v.as_str())
            .collect();
        assert_eq!(lts_majors.first(), Some(&"22.11.0"));
    }
}
