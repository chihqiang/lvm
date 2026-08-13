use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::core::extract;
use crate::core::report::{flush_reports_to_stdout, report};

// ─── Timeout and buffer configuration ───

const CACHE_TTL: Duration = Duration::from_secs(300);
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const HTTP_READ_TIMEOUT: Duration = Duration::from_secs(120);
const HTTP_TOTAL_TIMEOUT: Duration = Duration::from_secs(300);
const PROGRESS_BAR_TEMPLATE: &str = "[{bar:20}] {percent:3}%  {bytes} / {total_bytes}";
const PROGRESS_BAR_CHARS: &str = "=>-";

// ─── Offline mode ───

static OFFLINE: AtomicBool = AtomicBool::new(false);

static PARALLEL_DOWNLOADS: AtomicBool = AtomicBool::new(false);

pub fn set_offline(offline: bool) {
    OFFLINE.store(offline, Ordering::Release);
}

pub fn is_offline() -> bool {
    OFFLINE.load(Ordering::Relaxed)
}

pub fn set_parallel_downloads(parallel: bool) {
    PARALLEL_DOWNLOADS.store(parallel, Ordering::Release);
}

fn show_download_progress(requested: bool) -> bool {
    requested && !PARALLEL_DOWNLOADS.load(Ordering::Relaxed)
}

// ─── HTTP client ───

fn user_agent() -> &'static str {
    static UA: OnceLock<String> = OnceLock::new();
    UA.get_or_init(|| format!("lvm-http-client/{}", env!("CARGO_PKG_VERSION")))
}

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(HTTP_CONNECT_TIMEOUT)
            .timeout_read(HTTP_READ_TIMEOUT)
            .timeout(HTTP_TOTAL_TIMEOUT)
            .build()
    })
}

pub fn get_url(url: &str) -> ureq::Request {
    agent().get(url).set("User-Agent", user_agent())
}

// ─── Download and installation ───

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
    download(dl_url, tar_path, show_download_progress(true))?;
    verify(tar_path)
}

pub fn download_and_install(
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
    if let Ok(style) = ProgressStyle::default_bar().template(PROGRESS_BAR_TEMPLATE) {
        bar.set_style(style.progress_chars(PROGRESS_BAR_CHARS));
    }
    if position > 0 {
        bar.set_position(position);
    }
    bar
}

pub fn download(url: &str, dest: &Path, show_progress: bool) -> Result<()> {
    if is_offline() {
        return handle_offline(dest);
    }

    let existing = fs::metadata(dest).map_or(0, |m| m.len());
    let (resp, is_resume, total) = prepare_download(url, existing)?;
    // 416 Range Not Satisfiable means the existing file is already complete;
    // leave it untouched instead of truncating and rewriting it.
    if resp.status() == 416 {
        return Ok(());
    }
    let mut file = open_dest_file(dest, is_resume, existing)?;

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
    let downloaded = write_stream(&mut reader, &mut file, init_pos, &pb)?;

    finish_progress(pb, downloaded);
    report("Download complete");

    Ok(())
}

pub fn fetch_with_cache(
    cache_file: &Path,
    fetch_fn: impl FnOnce() -> Result<String>,
) -> Result<String> {
    fetch_with_cache_ttl(cache_file, CACHE_TTL, fetch_fn)
}

/// Like [`fetch_with_cache`], but with an explicit cache TTL.
pub fn fetch_with_cache_ttl(
    cache_file: &Path,
    ttl: Duration,
    fetch_fn: impl FnOnce() -> Result<String>,
) -> Result<String> {
    if let Ok(meta) = fs::metadata(cache_file)
        && let Ok(modified) = meta.modified()
        && let Ok(elapsed) = modified.elapsed()
        && elapsed < ttl
    {
        return fs::read_to_string(cache_file).context("Failed to read cache");
    }

    let text = fetch_fn()?;

    if let Some(parent) = cache_file.parent() {
        fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }
    // Atomic write: write to temp file, then rename to prevent partial writes
    let tmp = cache_file.with_extension("tmp");
    fs::write(&tmp, &text).context("Failed to write cache")?;
    fs::rename(&tmp, cache_file).context("Failed to finalize cache")?;

    Ok(text)
}

/// Handle offline mode: return Ok if file exists, otherwise bail.
fn handle_offline(dest: &Path) -> Result<()> {
    if dest.exists() {
        report("Using cached file (offline mode)");
        return Ok(());
    }
    bail!("Offline mode: no cached file at {}", dest.display())
}

/// Send the HTTP request with optional Range header and validate the response.
/// Returns (response, is_resume, total_bytes).
fn prepare_download(url: &str, existing: u64) -> Result<(ureq::Response, bool, u64)> {
    let mut req = get_url(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={existing}-"));
    }

    let resp = req.call().context("Download request failed")?;
    let status = resp.status();

    // 416 Range Not Satisfiable: file is already fully downloaded
    if status == 416 && existing > 0 {
        report("File already fully downloaded");
        // Return a dummy response; caller should check this status
        return Ok((resp, false, 0));
    }

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

    Ok((resp, is_resume, total))
}

/// Open the destination file for writing, appending if resuming.
fn open_dest_file(dest: &Path, is_resume: bool, _existing: u64) -> Result<fs::File> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(is_resume)
        .open(dest)
        .context("Failed to open file")
}

/// Read data from the HTTP response and write it to the file, updating the
/// progress bar as we go. Returns the total number of bytes downloaded.
fn write_stream(
    reader: &mut dyn std::io::Read,
    file: &mut dyn std::io::Write,
    init_pos: u64,
    pb: &Option<indicatif::ProgressBar>,
) -> Result<u64> {
    let mut buf = [0u8; 16384];
    let mut downloaded = init_pos;

    loop {
        let n = reader.read(&mut buf).context("Failed to read data")?;
        if n == 0 {
            return Ok(downloaded);
        }
        file.write_all(&buf[..n]).context("Failed to write file")?;
        downloaded += n as u64;
        if let Some(bar) = pb {
            bar.set_position(downloaded);
        }
    }
}

fn finish_progress(pb: Option<indicatif::ProgressBar>, _downloaded: u64) {
    if let Some(bar) = pb {
        bar.finish_and_clear();
    }
}

pub fn fetch_from_mirror(mirror_url: &str, url_path: &str) -> Result<String> {
    let url = format!(
        "{}/{}",
        mirror_url.trim_end_matches('/'),
        url_path.trim_start_matches('/')
    );
    let response = get_url(&url)
        .call()
        .context("Failed to fetch data from mirror")?;
    response.into_string().context("Failed to read response")
}
