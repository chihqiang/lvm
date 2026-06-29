#[test]
#[serial_test::serial]
fn test_report_and_drain() {
    let _ = lvm::core::report::drain_reports();
    lvm::core::report::report("test message 1");
    lvm::core::report::report("test message 2");

    let reports = lvm::core::report::drain_reports();
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0], "test message 1");
    assert_eq!(reports[1], "test message 2");
}

#[test]
fn test_drain_empty() {
    let _ = lvm::core::report::drain_reports();
    let reports = lvm::core::report::drain_reports();
    assert!(reports.is_empty());
}

#[test]
#[serial_test::serial]
fn test_report_functions_format() {
    let _ = lvm::core::report::drain_reports();
    lvm::core::report::report_verifying_checksum();
    lvm::core::report::report_checksum_verified();
    lvm::core::report::report_already_installed("node", "22.0.0");
    lvm::core::report::report_non_native_arch("linux", "arm64");
    lvm::core::report::report_fallback("linux-arm64", "linux-x64");

    let reports = lvm::core::report::drain_reports();
    assert_eq!(reports[0], "Verifying checksum...");
    assert_eq!(reports[1], "Checksum verified");
    assert_eq!(reports[2], "node 22.0.0 is already installed");
    assert_eq!(reports[3], "Using linux-arm64 (non-native arch)");
    assert_eq!(
        reports[4],
        "Failed for linux-arm64, falling back to linux-x64"
    );
}
