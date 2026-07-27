#[test]
fn test_offline_toggle() {
    assert!(!lvm::core::http::is_offline());

    lvm::core::http::set_offline(true);
    assert!(lvm::core::http::is_offline());

    lvm::core::http::set_offline(false);
    assert!(!lvm::core::http::is_offline());
}

#[test]
fn test_parallel_downloads_toggle() {
    lvm::core::http::set_parallel_downloads(true);
    lvm::core::http::set_parallel_downloads(false);
}
