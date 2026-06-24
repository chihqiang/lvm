use std::sync::{Mutex, OnceLock};

static REPORT_BUF: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn report_buf() -> &'static Mutex<Vec<String>> {
    REPORT_BUF.get_or_init(|| Mutex::new(Vec::new()))
}

fn lock_report_buf() -> std::sync::MutexGuard<'static, Vec<String>> {
    report_buf().lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn report(msg: impl Into<String>) {
    lock_report_buf().push(msg.into());
}

pub(crate) fn drain_reports() -> Vec<String> {
    lock_report_buf().drain(..).collect()
}

pub(crate) fn flush_reports_to_stdout() {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    for msg in drain_reports() {
        let _ = writeln!(out, "{msg}");
    }
    let _ = out.flush();
}
