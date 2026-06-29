use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config;

static OFFLINE: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_offline(offline: bool) {
    OFFLINE.store(offline, Ordering::Release);
}

pub(crate) fn is_offline() -> bool {
    OFFLINE.load(Ordering::Relaxed)
}

pub(crate) fn http() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(config::http_connect_timeout())
            .timeout_read(config::http_read_timeout())
            .timeout(config::http_total_timeout())
            .build()
    })
}

pub(crate) fn get_url(url: &str) -> ureq::Request {
    let ua = format!("lvm-http-client/{}", env!("CARGO_PKG_VERSION"));
    http().get(url).set("User-Agent", &ua)
}
