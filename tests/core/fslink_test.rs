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

    // Create symlink
    lvm::core::fslink::create_symlink(&target, &link).unwrap();
    assert!(link.exists());
    assert_eq!(fs::read_to_string(&link).unwrap(), "content");

    // Atomically replace symlink
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

#[test]
fn test_uninstall_version_removes_extra_links_when_current() {
    use lvm::core::fslink;

    let dir = tempfile::tempdir().unwrap();
    let version_dir = dir.path().join("v1.0.0");
    let bin_dir = version_dir.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(bin_dir.join("node"), "binary").unwrap();

    let current_link = dir.path().join("current");
    let bin_link = dir.path().join("node");
    let extra_link = dir.path().join("cargo");

    fslink::create_symlink(&version_dir, &current_link).unwrap();
    fslink::create_symlink(&bin_dir.join("node"), &bin_link).unwrap();
    fslink::create_symlink(&bin_dir.join("node"), &extra_link).unwrap();

    // Uninstall the CURRENT version: must clean current/bin/extra links.
    fslink::uninstall_version(
        &version_dir,
        &current_link,
        &bin_link,
        std::slice::from_ref(&extra_link),
        Some("1.0.0"),
        "1.0.0",
    )
    .unwrap();

    assert!(!current_link.exists());
    assert!(!bin_link.exists());
    assert!(!extra_link.exists()); // Bug5 regression: extra links are cleaned up
    assert!(!version_dir.exists());
}

#[test]
fn test_uninstall_version_keeps_links_when_not_current() {
    use lvm::core::fslink;

    let dir = tempfile::tempdir().unwrap();
    let version_dir = dir.path().join("v2.0.0");
    let bin_dir = version_dir.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(bin_dir.join("node"), "binary").unwrap();

    let current_link = dir.path().join("current");
    let bin_link = dir.path().join("node");
    let extra_link = dir.path().join("cargo");
    fslink::create_symlink(&version_dir, &current_link).unwrap();
    fslink::create_symlink(&bin_dir.join("node"), &bin_link).unwrap();

    // Uninstalling a NON-current version must NOT remove any links.
    fslink::uninstall_version(
        &version_dir,
        &current_link,
        &bin_link,
        std::slice::from_ref(&extra_link),
        Some("1.0.0"), // current is 1.0.0, not 2.0.0
        "2.0.0",
    )
    .unwrap();

    // Use is_symlink(): exists() would follow the (now-deleted) target dir.
    assert!(current_link.is_symlink());
    assert!(bin_link.is_symlink());
    assert!(!extra_link.is_symlink()); // never created, still absent
    assert!(!version_dir.exists());
}

#[test]
fn test_uninstall_version_not_installed_errors() {
    use lvm::core::fslink;

    let dir = tempfile::tempdir().unwrap();
    let version_dir = dir.path().join("v9.9.9");
    let result = fslink::uninstall_version(
        &version_dir,
        &dir.path().join("current"),
        &dir.path().join("node"),
        &[],
        None,
        "9.9.9",
    );
    assert!(result.is_err());
}
