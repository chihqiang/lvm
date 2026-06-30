pub mod dart;
pub mod flutter;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod node;
pub mod python;

mod language_trait;
mod registry;

pub(crate) use language_trait::Language;
pub(crate) use lvm::core::{
    CURRENT_DEFAULT_MARKER, CURRENT_MARKER, DEFAULT_MARKER, archive_ext, download_and_install,
    exe_suffix, fetch_from_mirror, fetch_with_cache, flush_reports_to_stdout, get_url,
    path_separator, remove_symlink, report, report_already_installed, report_checksum_verified,
    report_fallback, report_non_native_arch, report_verifying_checksum, resolve_partial_version,
    set_offline, sort_versions, verify_sha256,
};
pub(crate) use registry::LanguageRegistry;
