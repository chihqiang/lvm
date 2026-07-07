use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};

use super::version;

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

pub(crate) fn get_lts_info() -> Result<LtsInfo> {
    let text = version::fetch_index_tab()?;
    let ordered = parse_lts_info(&text);

    let mut name_to_ver: HashMap<String, String> = HashMap::new();

    for (ver, lts) in &ordered {
        if let Some(codename) = lts {
            let lower = codename.to_lowercase();
            name_to_ver.insert(lower, ver.clone());
        }
    }

    let latest = ordered
        .iter()
        .rev()
        .find(|(_, lts)| lts.is_some())
        .map(|(v, _)| v.clone());

    Ok(LtsInfo {
        latest,
        name_to_ver,
        ordered,
    })
}

pub(crate) fn resolve_lts(desc: &str) -> Result<String> {
    let info = get_lts_info()?;

    if desc == "*" || desc.is_empty() {
        return info.latest.ok_or_else(|| anyhow!("No LTS version found"));
    }

    if let Some(offset_str) = desc.strip_prefix('-')
        && let Ok(n) = offset_str.parse::<usize>()
    {
        let mut lts_versions: Vec<&str> = info
            .ordered
            .iter()
            .rev()
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
