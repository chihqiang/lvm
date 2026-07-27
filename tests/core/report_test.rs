#[test]
#[serial_test::serial]
fn test_report_and_flush() {
    // Flush any pending reports first
    lvm::core::report::flush_reports_to_stdout();

    lvm::core::report::report("test message 1");
    lvm::core::report::report("test message 2");

    // flush_reports_to_stdout should drain the buffer without panic
    lvm::core::report::flush_reports_to_stdout();

    // After flush, a second flush should be a no-op (no panic)
    lvm::core::report::flush_reports_to_stdout();
}

#[test]
#[serial_test::serial]
fn test_flush_empty() {
    // Flush any pending reports
    lvm::core::report::flush_reports_to_stdout();
    // Second flush on empty buffer should not panic
    lvm::core::report::flush_reports_to_stdout();
}

#[test]
#[serial_test::serial]
fn test_report_functions_format() {
    // Flush any pending reports first
    lvm::core::report::flush_reports_to_stdout();

    lvm::core::report::report_verifying_checksum();
    lvm::core::report::report_checksum_verified();
    lvm::core::report::report_already_installed("node", "22.0.0");
    lvm::core::report::report_non_native_arch("linux", "arm64");
    lvm::core::report::report_fallback("linux-arm64", "linux-x64");

    // flush_reports_to_stdout should drain all without panic
    lvm::core::report::flush_reports_to_stdout();
}
