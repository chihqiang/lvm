use serial_test::serial;

#[test]
#[serial]
fn test_offline_toggle() {
    assert!(!lvm::core::http::is_offline());

    lvm::core::http::set_offline(true);
    assert!(lvm::core::http::is_offline());

    lvm::core::http::set_offline(false);
    assert!(!lvm::core::http::is_offline());
}

#[test]
#[serial]
fn test_parallel_downloads_toggle() {
    lvm::core::http::set_parallel_downloads(true);
    lvm::core::http::set_parallel_downloads(false);
}

// ─── fetch_with_cache ───

/// Write `content` to `path` and force its mtime to `time` (needed because a
/// freshly-created file may not be considered "older" than the TTL).
fn write_cache_file(path: &std::path::Path, content: &str, mtime: std::time::SystemTime) {
    std::fs::write(path, content).unwrap();
    let f = std::fs::File::options().write(true).open(path).unwrap();
    f.set_modified(mtime).unwrap();
}

#[test]
fn test_fetch_with_cache_hit_does_not_call_fetch_fn() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.txt");
    // Fresh cache (mtime = now) → should be served from disk.
    write_cache_file(&cache, "cached-value", std::time::SystemTime::now());

    let result = lvm::core::http::fetch_with_cache(&cache, || {
        panic!("fetch_fn should NOT be called on a cache hit")
    });
    assert_eq!(result.unwrap(), "cached-value");
}

#[test]
fn test_fetch_with_cache_stale_refreshes_and_writes() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.txt");
    let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    write_cache_file(&cache, "stale", past);

    let result = lvm::core::http::fetch_with_cache(&cache, || Ok("fresh".to_string())).unwrap();
    assert_eq!(result, "fresh");
    // Cache file was refreshed.
    assert_eq!(std::fs::read_to_string(&cache).unwrap(), "fresh");
}

#[test]
fn test_fetch_with_cache_ttl_custom() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.txt");
    // 10 minutes old: within a 1-hour TTL (cache hit) but outside a 5-minute TTL.
    let past = std::time::SystemTime::now() - std::time::Duration::from_secs(600);
    write_cache_file(&cache, "old", past);

    let ttl = std::time::Duration::from_secs(3600);
    let hit = lvm::core::http::fetch_with_cache_ttl(&cache, ttl, || {
        panic!("should hit cache within TTL")
    })
    .unwrap();
    assert_eq!(hit, "old");

    let ttl_short = std::time::Duration::from_secs(60);
    let miss =
        lvm::core::http::fetch_with_cache_ttl(&cache, ttl_short, || Ok("new".to_string())).unwrap();
    assert_eq!(miss, "new");
}

// ─── download: 416 handling (Bug3 regression) ───

/// Spawn a one-shot TCP server that responds with the given status line.
fn serve_once(status_line: &str) -> std::net::SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let status_line = status_line.to_owned(); // owned for 'static move
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read, Write};
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!("{status_line}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    addr
}

#[test]
#[serial]
fn test_download_416_does_not_truncate_existing_file() {
    lvm::core::http::set_offline(false);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("file.bin");
    let original = b"existing-complete-data";
    std::fs::write(&dest, original).unwrap();

    let addr = serve_once("HTTP/1.1 416 Range Not Satisfiable");
    let result = lvm::core::http::download(&format!("http://{addr}/file.bin"), &dest, false);

    // Bug3: a 416 (file already complete) must not truncate the file.
    assert!(result.is_ok());
    assert_eq!(std::fs::read(&dest).unwrap(), original);
}

#[test]
#[serial]
fn test_download_offline_mode() {
    lvm::core::http::set_offline(true);
    let dir = tempfile::tempdir().unwrap();

    // File exists → Ok in offline mode.
    let dest = dir.path().join("cached.bin");
    std::fs::write(&dest, b"data").unwrap();
    assert!(lvm::core::http::download("http://example.invalid/x", &dest, false).is_ok());

    // File missing → error in offline mode.
    let missing = dir.path().join("missing.bin");
    assert!(lvm::core::http::download("http://example.invalid/x", &missing, false).is_err());

    lvm::core::http::set_offline(false);
}
