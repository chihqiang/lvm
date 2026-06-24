use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::config;
use crate::plugin::http::{get_url, is_offline};
use crate::plugin::report::{flush_reports_to_stdout, report};
use anyhow::{Context, Result, bail};

pub(crate) fn download_and_install(
    dl_url: &str,
    tar_path: &Path,
    version: &str,
    version_dir: &Path,
    display_name: &str,
    verify: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    if !tar_path.exists() {
        fs::create_dir_all(tar_path.parent().context("Invalid tar path")?)
            .context("Failed to create download cache directory")?;
        report(format!("Downloading {display_name} {version}"));
        report(format!("  from: {dl_url}"));
        report(format!("  to:   {}", tar_path.display()));
        flush_reports_to_stdout();
        download(dl_url, tar_path, true)?;
        verify(tar_path)?;
    }
    extract_archive(tar_path, version_dir)?;
    report(format!("{display_name} {version} installed successfully!"));
    Ok(())
}

pub(crate) fn download(url: &str, dest: &Path, show_progress: bool) -> Result<()> {
    if is_offline() {
        if dest.exists() {
            report("Using cached file (offline mode)");
            return Ok(());
        }
        bail!("Offline mode: no cached file at {}", dest.display())
    }

    let existing = fs::metadata(dest).ok().map_or(0, |m| m.len());

    let mut req = get_url(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={existing}-"));
    }

    let resp = req.call().context("Download request failed")?;
    let status = resp.status();
    if status != 200 && status != 206 {
        bail!("Download failed (HTTP {status})")
    }
    let is_resume = status == 206;

    let content_len = resp
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let total = if is_resume {
        existing + content_len
    } else if content_len > 0 {
        content_len
    } else {
        0
    };

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(is_resume)
        .open(dest)
        .context("Failed to open file")?;

    if !is_resume && existing > 0 {
        file.set_len(0).context("Failed to truncate file")?;
    }

    let pb = if show_progress {
        use indicatif::{ProgressBar, ProgressStyle};
        let bar = if total > 0 {
            ProgressBar::new(total)
        } else {
            ProgressBar::new_spinner()
        };
        if let Ok(style) = ProgressStyle::default_bar().template(config::progress_bar_template()) {
            bar.set_style(style.progress_chars(config::progress_bar_chars()));
        }
        if is_resume {
            bar.set_position(existing);
        }
        Some(bar)
    } else {
        None
    };

    let mut reader = resp.into_reader();
    let mut buf = vec![0u8; config::download_buffer_size()];
    let mut downloaded = if is_resume { existing } else { 0 };

    loop {
        let n = reader.read(&mut buf).context("Failed to read data")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).context("Failed to write file")?;
        downloaded += n as u64;

        if let Some(ref bar) = pb {
            bar.set_position(downloaded);
        }
    }

    if let Some(bar) = pb {
        bar.finish_and_clear();
    }
    report("Download complete");

    Ok(())
}

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
    let actual = sha256_of(file_path)?;
    if actual != expected_hex {
        bail!(
            "Checksum mismatch for {}: expected {expected_hex}, got {actual}",
            file_path.display()
        )
    }
    Ok(())
}

pub(crate) fn fetch_from_mirror(mirror_url: &str, url_path: &str) -> Result<String> {
    let url = format!("{}/{url_path}", mirror_url);
    let response = get_url(&url)
        .call()
        .context("Failed to fetch data from mirror")?;
    response.into_string().context("Failed to read response")
}

fn extract_zip(zip_path: &Path, version_dir: &Path) -> Result<()> {
    report("Extracting...");
    let file = fs::File::open(zip_path).context("Failed to open archive")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context("Extraction failed")?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };

        if path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            bail!("Zip entry contains path traversal (..)")
        }

        let mut out_path = version_dir.to_path_buf();
        for component in path.components().skip(1) {
            out_path.push(component);
        }

        if entry.is_dir() {
            fs::create_dir_all(&out_path).context("Extraction failed")?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).context("Extraction failed")?;
            }
            let mut outfile = fs::File::create(&out_path).context("Extraction failed")?;
            std::io::copy(&mut entry, &mut outfile).context("Extraction failed")?;
        }
    }
    Ok(())
}

fn extract_tarball(tar_path: &Path, version_dir: &Path) -> Result<()> {
    report("Extracting...");
    let tar_file = fs::File::open(tar_path).context("Failed to open tarball")?;
    let decoder = flate2::read::GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(decoder);
    let result = archive
        .entries()
        .context("Extraction failed")?
        .try_for_each(|entry| {
            let mut entry = entry.context("Extraction failed")?;
            let path = entry.path().context("Extraction failed")?;
            if path.is_absolute() {
                bail!("Extraction failed: absolute path in archive")
            }
            let stripped = path.components().skip(1).collect::<std::path::PathBuf>();
            if stripped.as_os_str().is_empty() {
                return Ok(());
            }
            if stripped
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                bail!("Extraction failed: path traversal in archive");
            }
            let out_path = version_dir.join(&stripped);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).context("Extraction failed")?;
            }
            entry
                .unpack(&out_path)
                .map(|_| ())
                .context("Extraction failed")
        });

    if let Err(err) = result {
        let _ = fs::remove_dir_all(version_dir);
        let _ = fs::remove_file(tar_path);
        bail!("{}", err)
    }
    Ok(())
}

pub(crate) fn extract_archive(archive_path: &Path, version_dir: &Path) -> Result<()> {
    if archive_path.extension().is_some_and(|e| e == "zip") {
        extract_zip(archive_path, version_dir)
    } else {
        extract_tarball(archive_path, version_dir)
    }
}
