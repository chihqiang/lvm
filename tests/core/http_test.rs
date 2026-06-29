#[test]
fn test_offline_toggle() {
    assert!(!lvm::core::http::is_offline());

    lvm::core::http::set_offline(true);
    assert!(lvm::core::http::is_offline());

    lvm::core::http::set_offline(false);
    assert!(!lvm::core::http::is_offline());
}
