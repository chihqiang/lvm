use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::config;
use crate::language::http::get_url;
use anyhow::{Context, Result, bail};

pub(crate) fn sha256_of(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).context("Failed to open file for checksum")?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; config::download_buffer_size()];
    loop {
        let n = file.read(&mut buf).context("Failed to read for checksum")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn fetch_checksums(mirror_url: &str, version: &str) -> Result<HashMap<String, String>> {
    let url = format!("{mirror_url}/v{version}/SHASUMS256.txt");
    let response = get_url(&url).call().context("Failed to fetch checksums")?;
    let text = response.into_string().context("Failed to read checksums")?;

    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((checksum, filename)) = line.split_once("  ") {
            map.insert(filename.trim().to_string(), checksum.trim().to_string());
        } else if let Some((checksum, filename)) = line.split_once(' ') {
            map.insert(filename.trim().to_string(), checksum.trim().to_string());
        }
    }
    Ok(map)
}

pub(crate) fn verify_sha256(file_path: &Path, expected_hex: &str) -> Result<()> {
    let expected_hex = expected_hex.trim();
    if expected_hex.len() != 64 || !expected_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Invalid SHA256 checksum for {}", file_path.display());
    }
    let actual = sha256_of(file_path)?;
    if !actual.eq_ignore_ascii_case(expected_hex) {
        bail!(
            "Checksum mismatch for {}: expected {expected_hex}, got {actual}",
            file_path.display()
        )
    }
    Ok(())
}
