use lvm::language::node::NodeLanguage;

// Tests moved out of src/language/node/version.rs — the functions under test
// are exposed via the public API (NodeLanguage::parse_index_tab /
// NodeLanguage::version_from_url / lvm::language::node::version_from_tarball_name).

#[test]
fn parse_index_tab_skips_header_and_strips_v() {
    let text = "version\tdate\tfiles\nv22.0.0\t2024-10-01\t20\nv20.0.0\t2024-04-01\t20\n";
    let versions = NodeLanguage::parse_index_tab(text);
    assert_eq!(versions, vec!["22.0.0", "20.0.0"]);
}

#[test]
fn version_from_tarball_name_extracts_version() {
    assert_eq!(
        lvm::language::node::version_from_tarball_name("node-v22.3.1-linux-x64.tar.gz"),
        Some("22.3.1".to_string())
    );
    assert_eq!(
        lvm::language::node::version_from_tarball_name("foo.tar.gz"),
        None
    );
}

#[test]
fn version_from_url_parses_custom_url() {
    assert_eq!(
        NodeLanguage::version_from_url("https://example.com/node-v20.14.0-linux-x64.tar.gz")
            .unwrap(),
        "20.14.0"
    );
    assert!(NodeLanguage::version_from_url("https://example.com/not-node.tar.gz").is_err());
}

#[test]
fn version_from_url_rejects_invalid_semver() {
    assert!(
        NodeLanguage::version_from_url("https://example.com/node-vabc-linux-x64.tar.gz").is_err()
    );
}
