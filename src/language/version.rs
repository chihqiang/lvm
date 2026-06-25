use semver::Version;

use anyhow::{Result, anyhow, bail};

pub(crate) fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_ver = Version::parse(a).ok();
    let b_ver = Version::parse(b).ok();
    match (a_ver, b_ver) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        _ => a.cmp(b),
    }
}

pub(crate) fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_versions(a, b));
}

pub(crate) fn resolve_partial_version(
    candidate: &str,
    avail: &[String],
    lang: &str,
) -> Result<String> {
    let parts: Vec<&str> = candidate.split('.').collect();
    let want_major = parts.first().and_then(|s| s.parse::<u64>().ok());
    if want_major.is_none() {
        bail!("Invalid version '{candidate}' for {lang}");
    }
    let want_minor = parts.get(1).and_then(|s| s.parse::<u64>().ok());

    let best = avail
        .iter()
        .filter_map(|a| Version::parse(a).ok())
        .filter(|av| {
            want_major.is_none_or(|maj| av.major == maj)
                && want_minor.is_none_or(|min| av.minor == min)
        })
        .max();
    best.map(|v| v.to_string())
        .ok_or_else(|| anyhow!("Could not find {lang} version matching: {candidate}"))
}
