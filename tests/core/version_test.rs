use semver::Version;

fn v(ver: &str) -> Version {
    Version::parse(ver).unwrap()
}

#[test]
fn test_compare_versions_equal() {
    use std::cmp::Ordering;
    assert_eq!(
        lvm::core::version::compare_versions("1.0.0", "1.0.0"),
        Ordering::Equal
    );
}

#[test]
fn test_compare_versions_greater() {
    use std::cmp::Ordering;
    assert_eq!(
        lvm::core::version::compare_versions("2.0.0", "1.0.0"),
        Ordering::Greater
    );
}

#[test]
fn test_compare_versions_less() {
    use std::cmp::Ordering;
    assert_eq!(
        lvm::core::version::compare_versions("1.0.0", "2.0.0"),
        Ordering::Less
    );
}

#[test]
fn test_compare_versions_with_pre_release() {
    use std::cmp::Ordering;
    assert_eq!(
        lvm::core::version::compare_versions("1.0.0-alpha", "1.0.0"),
        Ordering::Less
    );
}

#[test]
fn test_compare_versions_non_semver() {
    use std::cmp::Ordering;
    // non-semver strings compare lexicographically
    assert_eq!(
        lvm::core::version::compare_versions("nightly", "stable"),
        Ordering::Less
    );
}

#[test]
fn test_sort_versions() {
    let mut versions = vec![
        "2.0.0".to_string(),
        "1.0.0".to_string(),
        "3.0.0".to_string(),
        "1.5.0".to_string(),
    ];
    lvm::core::version::sort_versions(&mut versions);
    assert_eq!(versions, vec!["1.0.0", "1.5.0", "2.0.0", "3.0.0"]);
}

#[test]
fn test_sort_versions_with_non_semver() {
    let mut versions = vec![
        "22.0.0".to_string(),
        "nightly".to_string(),
        "20.0.0".to_string(),
    ];
    lvm::core::version::sort_versions(&mut versions);
    // semver versions sorted, non-semver sorted lexicographically afterwards
    assert_eq!(versions, vec!["nightly", "20.0.0", "22.0.0"]);
}

#[test]
fn test_resolve_partial_version_exact() {
    let avail = vec![v("22.0.0"), v("20.0.0"), v("18.0.0")];
    let result = lvm::core::version::resolve_partial_version("22", &avail, "node").unwrap();
    assert_eq!(result, "22.0.0");
}

#[test]
fn test_resolve_partial_version_latest_minor() {
    let avail = vec![v("22.3.0"), v("22.1.0"), v("22.2.0")];
    let result = lvm::core::version::resolve_partial_version("22", &avail, "node").unwrap();
    assert_eq!(result, "22.3.0");
}

#[test]
fn test_resolve_partial_version_major_minor() {
    let avail = vec![v("22.3.1"), v("22.3.0"), v("22.2.0")];
    let result = lvm::core::version::resolve_partial_version("22.3", &avail, "node").unwrap();
    assert_eq!(result, "22.3.1");
}

#[test]
fn test_resolve_partial_version_no_match() {
    let avail = vec![v("20.0.0"), v("18.0.0")];
    let result = lvm::core::version::resolve_partial_version("22", &avail, "node");
    assert!(result.is_err());
}

#[test]
fn test_resolve_partial_version_invalid_input() {
    let avail = vec![v("20.0.0")];
    let result = lvm::core::version::resolve_partial_version("abc", &avail, "node");
    assert!(result.is_err());
}

#[test]
fn test_parse_github_releases_valid() {
    let json = r#"[
        {"tag_name": "v1.0.0", "draft": false, "prerelease": false},
        {"tag_name": "v1.1.0", "draft": false, "prerelease": false},
        {"tag_name": "v2.0.0-rc1", "draft": false, "prerelease": true}
    ]"#;
    let versions = lvm::core::version::parse_github_releases(json).unwrap();
    assert_eq!(versions, vec!["1.0.0", "1.1.0"]);
}

#[test]
fn test_parse_github_releases_invalid_json() {
    let result = lvm::core::version::parse_github_releases("not json");
    assert!(result.is_err());
}
