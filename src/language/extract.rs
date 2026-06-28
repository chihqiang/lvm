use std::fs;
use std::path::{Path, PathBuf};

use crate::language::report::report;
use anyhow::{Context, Result, bail};

fn strip_top_level(path: &Path) -> Result<PathBuf> {
    let stripped: PathBuf = path.components().skip(1).collect();
    if stripped
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        bail!("Extraction failed: path traversal in archive")
    }
    Ok(stripped)
}

fn cleanup_on_failure(version_dir: &Path, result: Result<()>) -> Result<()> {
    if let Err(err) = result {
        if let Err(e) = fs::remove_dir_all(version_dir) {
            report(format!(
                "Warning: failed to cleanup {}: {e}",
                version_dir.display()
            ));
        }
        bail!("{err}")
    }
    Ok(())
}

fn extract_zip(zip_path: &Path, version_dir: &Path) -> Result<()> {
    report("Extracting...");
    let file = fs::File::open(zip_path).context("Failed to open archive")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    let result = (|| -> Result<()> {
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).context("Extraction failed")?;
            let Some(path) = entry.enclosed_name() else {
                continue;
            };

            let out_path = version_dir.join(strip_top_level(&path)?);

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
    })();

    cleanup_on_failure(version_dir, result)
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
            let stripped = strip_top_level(&path)?;
            if stripped.as_os_str().is_empty() {
                return Ok(());
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

    cleanup_on_failure(version_dir, result)
}

pub(crate) fn extract_archive(archive_path: &Path, version_dir: &Path) -> Result<()> {
    if archive_path.extension().is_some_and(|e| e == "zip") {
        extract_zip(archive_path, version_dir)
    } else {
        extract_tarball(archive_path, version_dir)
    }
}
