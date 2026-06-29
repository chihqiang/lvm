use std::io::Write;
use std::sync::{Mutex, OnceLock};

static REPORT_BUF: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn report_buf() -> &'static Mutex<Vec<String>> {
    REPORT_BUF.get_or_init(|| Mutex::new(Vec::new()))
}

fn lock_report_buf() -> std::sync::MutexGuard<'static, Vec<String>> {
    match report_buf().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(crate) fn report(msg: impl Into<String>) {
    lock_report_buf().push(msg.into());
}

pub(crate) fn drain_reports() -> Vec<String> {
    lock_report_buf().drain(..).collect()
}

pub(crate) fn flush_reports_to_stdout() {
    let mut stdout = std::io::stdout().lock();
    for msg in drain_reports() {
        let _ = writeln!(stdout, "{msg}");
    }
    let _ = stdout.flush();
}

pub(crate) fn report_verifying_checksum() {
    report("Verifying checksum...");
}

pub(crate) fn report_checksum_verified() {
    report("Checksum verified");
}

pub(crate) fn report_already_installed(name: &str, version: &str) {
    report(format!("{name} {version} is already installed"));
}

pub(crate) fn report_non_native_arch(os: &str, arch: &str) {
    report(format!("Using {os}-{arch} (non-native arch)"));
}

pub(crate) fn report_fallback(from: &str, to: &str) {
    report(format!("Failed for {from}, falling back to {to}"));
}
