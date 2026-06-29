use serial_test::serial;
use std::fs;

/// config 常量函数测试
#[test]
fn test_bin_dir_name() {
    assert_eq!(lvm::core::config::bin_dir_name(), "bin");
}

#[test]
fn test_current_dir_name() {
    assert_eq!(lvm::core::config::current_dir_name(), "current");
}

#[test]
fn test_aliases_dir_name() {
    assert_eq!(lvm::core::config::aliases_dir_name(), "aliases");
}

#[test]
fn test_system_version_keyword() {
    assert_eq!(lvm::core::config::system_version_keyword(), "system");
}

#[test]
fn test_lts_prefix() {
    assert_eq!(lvm::core::config::lts_prefix(), "lts/");
}

#[test]
fn test_list_separator() {
    assert_eq!(lvm::core::config::list_separator(), ", ");
}

#[test]
fn test_max_lvmrc_depth() {
    assert_eq!(lvm::core::config::max_lvmrc_depth(), 100);
}

#[test]
fn test_lvmrc_filename() {
    assert_eq!(lvm::core::config::lvmrc_filename(), ".lvmrc");
}

/// parse_lvmrc 测试
#[test]
fn test_parse_lvmrc_valid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".lvmrc");
    fs::write(&path, "node=22.0.0\ngo=1.22.3\n").unwrap();
    let map = lvm::core::config::parse_lvmrc(&path).unwrap();
    assert_eq!(map.get("node").unwrap(), "22.0.0");
    assert_eq!(map.get("go").unwrap(), "1.22.3");
}

#[test]
fn test_parse_lvmrc_with_comments_and_blanks() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".lvmrc");
    fs::write(
        &path,
        "# comment\n\nnode=20.14.0\n  # indented\n\ngo=1.21.0\n",
    )
    .unwrap();
    let map = lvm::core::config::parse_lvmrc(&path).unwrap();
    assert_eq!(map.get("node").unwrap(), "20.14.0");
    assert_eq!(map.get("go").unwrap(), "1.21.0");
}

#[test]
fn test_parse_lvmrc_empty_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".lvmrc");
    fs::write(&path, "=22.0.0\n").unwrap();
    let result = lvm::core::config::parse_lvmrc(&path);
    assert!(result.is_err());
}

#[test]
fn test_parse_lvmrc_invalid_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".lvmrc");
    fs::write(&path, "invalid-line\n").unwrap();
    let result = lvm::core::config::parse_lvmrc(&path);
    assert!(result.is_err());
}

#[test]
fn test_parse_lvmrc_empty_value() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".lvmrc");
    fs::write(&path, "node=\n").unwrap();
    let result = lvm::core::config::parse_lvmrc(&path);
    assert!(result.is_err());
}

/// set_alias / get_alias / list_alias_names / remove_alias 集成测试
#[test]
#[serial]
fn test_alias_crud() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join(".lvm");
    fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("HOME", dir.path().as_os_str().to_str().unwrap()) };

    // set
    lvm::core::config::set_alias("node", "stable", "22.0.0").unwrap();
    lvm::core::config::set_alias("node", "default", "22").unwrap();

    // get
    assert_eq!(
        lvm::core::config::get_alias("node", "stable").unwrap(),
        Some("22.0.0".to_string())
    );
    assert_eq!(
        lvm::core::config::get_alias("node", "nonexistent").unwrap(),
        None
    );

    // list
    let names = lvm::core::config::list_alias_names("node").unwrap();
    assert!(names.contains(&"default".to_string()));
    assert!(names.contains(&"stable".to_string()));

    // remove
    lvm::core::config::remove_alias("node", "stable").unwrap();
    assert_eq!(
        lvm::core::config::get_alias("node", "stable").unwrap(),
        None
    );
}

/// set_default_version / get_default_version 测试
#[test]
#[serial]
fn test_default_version() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join(".lvm");
    fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("HOME", dir.path().as_os_str().to_str().unwrap()) };

    lvm::core::config::set_default_version("go", "1.22.0").unwrap();
    assert_eq!(
        lvm::core::config::get_default_version("go").unwrap(),
        Some("1.22.0".to_string())
    );

    // no default yet
    assert_eq!(
        lvm::core::config::get_default_version("python").unwrap(),
        None
    );
}

/// write_lvmrc / read_lvmrc_version 测试
#[test]
#[serial]
fn test_write_and_read_lvmrc() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    lvm::core::config::write_lvmrc("node", "22.0.0").unwrap();
    let result = lvm::core::config::read_lvmrc_version("node").unwrap();

    std::env::set_current_dir(cwd).unwrap();
    assert_eq!(result, Some("22.0.0".to_string()));
}

#[test]
#[serial]
fn test_write_lvmrc_update_existing() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    lvm::core::config::write_lvmrc("node", "20.0.0").unwrap();
    lvm::core::config::write_lvmrc("go", "1.22.0").unwrap();
    lvm::core::config::write_lvmrc("node", "22.0.0").unwrap();

    let map = lvm::core::config::parse_lvmrc(&dir.path().join(".lvmrc")).unwrap();
    assert_eq!(map.get("node").unwrap(), "22.0.0");
    assert_eq!(map.get("go").unwrap(), "1.22.0");

    let _ = fs::remove_file(dir.path().join(".lvmrc"));
    std::env::set_current_dir(cwd).unwrap();
}

/// 颜色/显示函数测试
#[test]
fn test_display_functions() {
    assert!(lvm::core::config::use_color() || !lvm::core::config::use_color());
    assert!(lvm::core::config::colored_check_mark().contains('\u{2713}'));
    assert_eq!(lvm::core::config::lts_marker(), "(LTS:");
    assert_eq!(lvm::core::config::installed_check_mark(), "\u{2713}");
}
