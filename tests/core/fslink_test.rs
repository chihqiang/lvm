use std::fs;

#[test]
fn test_exe_suffix() {
    let suffix = lvm::core::fslink::exe_suffix();
    #[cfg(windows)]
    assert_eq!(suffix, ".exe");
    #[cfg(not(windows))]
    assert_eq!(suffix, "");
}

#[test]
fn test_path_separator() {
    let sep = lvm::core::fslink::path_separator();
    #[cfg(windows)]
    assert_eq!(sep, ";");
    #[cfg(not(windows))]
    assert_eq!(sep, ":");
}

#[test]
fn test_archive_ext() {
    let ext = lvm::core::fslink::archive_ext();
    #[cfg(windows)]
    assert_eq!(ext, "zip");
    #[cfg(not(windows))]
    assert_eq!(ext, "tar.gz");
}

#[test]
fn test_create_and_replace_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    let link = dir.path().join("link");
    let new_target = dir.path().join("new_target");

    fs::write(&target, "content").unwrap();
    fs::write(&new_target, "new content").unwrap();

    // 创建符号链接
    lvm::core::fslink::create_symlink(&target, &link).unwrap();
    assert!(link.exists());
    assert_eq!(fs::read_to_string(&link).unwrap(), "content");

    // 原子替换符号链接
    lvm::core::fslink::replace_symlink(&new_target, &link).unwrap();
    assert_eq!(fs::read_to_string(&link).unwrap(), "new content");
}

#[test]
fn test_remove_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    let link = dir.path().join("link");

    fs::write(&target, "content").unwrap();
    lvm::core::fslink::create_symlink(&target, &link).unwrap();
    assert!(link.exists());

    lvm::core::fslink::remove_symlink(&link).unwrap();
    assert!(!link.exists());
}

#[test]
fn test_format_installed_versions() {
    let versions = vec![
        "22.0.0".to_string(),
        "20.0.0".to_string(),
        "18.0.0".to_string(),
    ];

    // no current or default
    let formatted = lvm::core::fslink::format_installed_versions("", None, None, &versions);
    assert_eq!(formatted, vec!["22.0.0", "20.0.0", "18.0.0"]);

    // with current
    let formatted =
        lvm::core::fslink::format_installed_versions("", Some("22.0.0"), None, &versions);
    assert_eq!(formatted[0], "22.0.0 (current)");
    assert_eq!(formatted[1], "20.0.0");

    // with current and default same
    let formatted =
        lvm::core::fslink::format_installed_versions("", Some("22.0.0"), Some("22.0.0"), &versions);
    assert_eq!(formatted[0], "22.0.0 (current, default)");

    // with prefix
    let formatted = lvm::core::fslink::format_installed_versions("v", None, None, &versions);
    assert_eq!(formatted[0], "v22.0.0");
}
