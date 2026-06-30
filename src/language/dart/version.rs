use anyhow::{Context, Result};
use quick_xml::de::from_str;
use serde::Deserialize;

use crate::config;
use crate::language;

use super::config::{
    dart_latest_version_cache_filename, dart_mirror, dart_versions_cache_filename,
};

#[derive(Debug, Deserialize)]
#[serde(rename = "ListBucketResult")]
struct S3ListResult {
    #[serde(rename = "Contents", default)]
    contents: Vec<S3Contents>,
    #[serde(rename = "IsTruncated")]
    is_truncated: bool,
    #[serde(rename = "NextContinuationToken")]
    next_continuation_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S3Contents {
    #[serde(rename = "Key")]
    key: String,
}

impl super::DartLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let cache_file = config::cache_path(dart_latest_version_cache_filename());

        let text = language::fetch_with_cache(&cache_file, || {
            let url = format!("{}/channels/stable/release/latest/VERSION", dart_mirror());
            let response = language::get_url(&url)
                .call()
                .context("Failed to fetch Dart latest version")?;
            response.into_string().context("Failed to read response")
        })?;

        let root: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse Dart latest version JSON")?;
        root.get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .context("Missing 'version' field in Dart latest version response")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_path(dart_versions_cache_filename());

        let text = language::fetch_with_cache(&cache_file, Self::fetch_s3_listing)?;

        let mut versions: Vec<semver::Version> = Vec::new();
        for line in text.lines() {
            if let Ok(ver) = semver::Version::parse(line)
                && ver.pre.is_empty()
            {
                versions.push(ver);
            }
        }
        versions.sort();
        versions.dedup();
        Ok(versions.into_iter().map(|v| v.to_string()).collect())
    }

    fn fetch_s3_listing() -> Result<String> {
        let mut all_versions: Vec<semver::Version> = Vec::new();
        let mut token: Option<String> = None;

        let base = format!("{}/", dart_mirror().trim_end_matches('/'));

        for _ in 0..100 {
            let mut req = language::get_url(&base)
                .query("list-type", "2")
                .query("prefix", "channels/stable/release/")
                .query("max-keys", "1000");
            if let Some(ref t) = token {
                req = req.query("continuation-token", t);
            }

            let response = req.call().context("Failed to fetch Dart releases")?;
            let xml = response.into_string().context("Failed to read response")?;

            let result: S3ListResult = from_str(&xml).context("Failed to parse S3 listing XML")?;

            for entry in &result.contents {
                if let Some(ver_str) = entry
                    .key
                    .strip_prefix("channels/stable/release/")
                    .and_then(|s| s.split('/').next())
                    && let Ok(ver) = semver::Version::parse(ver_str)
                    && ver.pre.is_empty()
                {
                    all_versions.push(ver);
                }
            }

            if !result.is_truncated {
                break;
            }

            match result.next_continuation_token {
                Some(t) if !t.is_empty() => token = Some(t),
                _ => break,
            }
        }

        all_versions.sort();
        all_versions.dedup();

        let mut buf = String::new();
        for v in &all_versions {
            buf.push_str(&v.to_string());
            buf.push('\n');
        }
        Ok(buf)
    }
}
