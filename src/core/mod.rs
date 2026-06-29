pub mod checksum;
pub mod config;
pub mod download;
pub mod extract;
pub mod fslink;
pub mod http;
pub mod report;
pub mod version;

pub use checksum::verify_sha256;
pub use download::{download_and_install, fetch_from_mirror, fetch_with_cache};
pub use fslink::{
    CURRENT_DEFAULT_MARKER, CURRENT_MARKER, DEFAULT_MARKER, archive_ext, exe_suffix,
    path_separator, remove_symlink,
};
pub use http::{get_url, set_offline};
pub use report::{
    flush_reports_to_stdout, report, report_already_installed, report_checksum_verified,
    report_fallback, report_non_native_arch, report_verifying_checksum,
};
pub use version::{resolve_partial_version, sort_versions};
