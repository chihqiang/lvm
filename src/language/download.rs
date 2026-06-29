use std::fs;
use std::io::Write;
use std::path::Path;

use crate::config;
use crate::language::extract;
use crate::language::http::{get_url, is_offline};
use crate::language::report::{flush_reports_to_stdout, report};
use anyhow::{Context, Result, bail};

fn install_temp_dir(version_dir: &Path) -> Result<std::path::PathBuf> {
    let parent = version_dir.parent().context("Invalid version directory")?;
    let name = version_dir
        .file_name()
        .context("Invalid version directory")?
        .to_string_lossy();
    Ok(parent.join(format!(".{name}.tmp-{}", std::process::id())))
}

fn cleanup_install_dir(path: &Path) {
    if path.exists()
        && let Err(e) = fs::remove_dir_all(path)
    {
        report(format!(
            "Warning: failed to cleanup {}: {e}",
            path.display()
        ));
    }
}

fn ensure_downloaded(
    dl_url: &str,
    tar_path: &Path,
    display_name: &str,
    version: &str,
    verify: impl Fn(&Path) -> Result<()>,
) -> Result<()> {
    if is_offline() {
        if tar_path.exists() {
            return Ok(());
        }
        bail!("Offline mode: no cached file at {}", tar_path.display())
    }

    if tar_path.exists() {
        if verify(tar_path).is_ok() {
            return Ok(());
        }
        report(format!(
            "Verification failed, re-downloading {display_name} {version}"
        ));
        fs::remove_file(tar_path)?;
    }

    fs::create_dir_all(tar_path.parent().context("Invalid tar path")?)
        .context("Failed to create download cache directory")?;
    report(format!("Downloading {display_name} {version}"));
    report(format!("  from: {dl_url}"));
    report(format!("  to:   {}", tar_path.display()));
    flush_reports_to_stdout();
    download(dl_url, tar_path, true)?;
    verify(tar_path)
}

pub(crate) fn download_and_install(
    dl_url: &str,
    tar_path: &Path,
    version: &str,
    version_dir: &Path,
    display_name: &str,
    verify: impl Fn(&Path) -> Result<()>,
) -> Result<()> {
    ensure_downloaded(dl_url, tar_path, display_name, version, verify)?;

    let temp_dir = install_temp_dir(version_dir)?;
    cleanup_install_dir(&temp_dir);
    fs::create_dir_all(
        temp_dir
            .parent()
            .context("Invalid temporary install path")?,
    )
    .context("Failed to create install directory")?;

    let result = (|| -> Result<()> {
        extract::extract_archive(tar_path, &temp_dir)?;
        if version_dir.exists() {
            fs::remove_dir_all(version_dir).with_context(|| {
                format!(
                    "Failed to replace incomplete install at {}",
                    version_dir.display()
                )
            })?;
        }
        fs::rename(&temp_dir, version_dir).with_context(|| {
            format!(
                "Failed to move {} into {}",
                temp_dir.display(),
                version_dir.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() {
        cleanup_install_dir(&temp_dir);
    }
    result?;

    report(format!("{display_name} {version} installed successfully!"));
    Ok(())
}

fn create_progress_bar(total: u64, position: u64) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};
    let bar = if total > 0 {
        ProgressBar::new(total)
    } else {
        ProgressBar::new_spinner()
    };
    if let Ok(style) = ProgressStyle::default_bar().template(config::progress_bar_template()) {
        bar.set_style(style.progress_chars(config::progress_bar_chars()));
    }
    if position > 0 {
        bar.set_position(position);
    }
    bar
}

pub(crate) fn download(url: &str, dest: &Path, show_progress: bool) -> Result<()> {
    if is_offline() {
        if dest.exists() {
            report("Using cached file (offline mode)");
            return Ok(());
        }
        bail!("Offline mode: no cached file at {}", dest.display())
    }

    let existing = fs::metadata(dest).map_or(0, |m| m.len());

    let mut req = get_url(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={existing}-"));
    }

    let resp = req.call().context("Download request failed")?;
    let status = resp.status();
    if status != 200 && status != 206 {
        let body = resp.into_string().unwrap_or_default();
        bail!("Download failed (HTTP {status}): {body}")
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

    let init_pos = if is_resume { existing } else { 0 };
    let pb = if show_progress {
        Some(create_progress_bar(total, init_pos))
    } else {
        None
    };

    let mut reader = resp.into_reader();
    let mut buf = [0u8; 16384];
    let mut downloaded = init_pos;

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

pub(crate) fn fetch_with_cache(
    cache_file: &Path,
    fetch_fn: impl FnOnce() -> Result<String>,
) -> Result<String> {
    if let Ok(meta) = fs::metadata(cache_file)
        && let Ok(modified) = meta.modified()
        && let Ok(elapsed) = modified.elapsed()
        && elapsed < config::cache_ttl()
    {
        return fs::read_to_string(cache_file).context("Failed to read cache");
    }

    let text = fetch_fn()?;

    if let Some(parent) = cache_file.parent() {
        fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }
    fs::write(cache_file, &text).context("Failed to write cache")?;

    Ok(text)
}

pub(crate) fn fetch_from_mirror(mirror_url: &str, url_path: &str) -> Result<String> {
    let url = format!("{}/{}", mirror_url.trim_end_matches('/'), url_path.trim_start_matches('/'));
    let response = get_url(&url)
        .call()
        .context("Failed to fetch data from mirror")?;
    response.into_string().context("Failed to read response")
}
