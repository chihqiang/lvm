#[test]
fn reject_system_install_accepts_regular_versions() {
    assert!(lvm::language::reject_system_install(None).is_ok());
    assert!(lvm::language::reject_system_install(Some("22.0.0")).is_ok());
}

#[test]
fn reject_system_install_rejects_system_keyword() {
    let err = lvm::language::reject_system_install(Some(" system ")).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Use 'lvm use system' instead of 'lvm install system'"
    );
}

#[test]
fn reject_lts_install_accepts_regular_versions() {
    assert!(lvm::language::reject_lts_install("Go", None).is_ok());
    assert!(lvm::language::reject_lts_install("Go", Some("1.22.0")).is_ok());
}

#[test]
fn reject_lts_install_rejects_lts_prefix() {
    let err = lvm::language::reject_lts_install("Go", Some(" lts/* ")).unwrap_err();
    assert_eq!(err.to_string(), "Go does not have LTS releases");
}
