use std::fs;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use anyhow::{Context, Result, bail};

pub(crate) fn sha256_of(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).context("Failed to open file for checksum")?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16384];
    loop {
        let n = file.read(&mut buf).context("Failed to read for checksum")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
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
