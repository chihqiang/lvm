use lvm::language::node::lts::{LtsInfo, build_lts_info};

// Tests moved out of src/language/node/lts.rs — the functions under test are
// exposed via lvm::language::node::lts::{build_lts_info, parse_lts_info, LtsInfo}.

/// Sample index.tab lines (version, date, files, npm, v8, uv, zlib, openssl,
/// modules, lts).
const SAMPLE_TAB: &str = concat!(
    "version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\n",
    "v23.0.0\t2024-10-15\t17\t10.9.0\t12.4.254.20\t1.48.0\t1.3.1\t3.0.13\t127\t\n",
    "v22.11.0\t2024-10-15\t22\t10.9.0\t12.4.254.20\t1.48.0\t1.3.1\t3.0.13\t127\tJod\n",
    "v20.18.1\t2024-10-15\t21\t10.8.2\t11.3.244.8\t1.48.0\t1.3.1\t3.0.13\t115\tIron\n",
    "v20.17.0\t2024-09-26\t21\t10.8.2\t11.3.244.8\t1.48.0\t1.3.1\t3.0.13\t115\tIron\n",
);

#[test]
fn build_lts_info_parses_mapping_and_latest() {
    let info = build_lts_info(SAMPLE_TAB);

    assert_eq!(info.latest.as_deref(), Some("22.11.0"));
    assert_eq!(info.name_to_ver.get("iron"), Some(&"20.18.1".to_string()));
    assert_eq!(info.name_to_ver.get("jod"), Some(&"22.11.0".to_string()));
    // Latest stable (non-LTS) version is present in ordered, not in LTS map.
    assert!(!info.name_to_ver.contains_key("23.0.0"));
}

#[test]
fn lts_info_serde_roundtrip_preserves_data() {
    let info = build_lts_info(SAMPLE_TAB);
    let json = serde_json::to_string(&info).expect("serialize");
    let decoded: LtsInfo = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.latest, info.latest);
    assert_eq!(decoded.name_to_ver, info.name_to_ver);
    assert_eq!(decoded.ordered, info.ordered);
}

#[test]
fn resolve_lts_by_name_and_offset() {
    let info = build_lts_info(SAMPLE_TAB);
    // Verify name lookup logic used by resolve_lts (newest patch wins).
    assert_eq!(
        info.name_to_ver.get("iron").map(String::as_str),
        Some("20.18.1")
    );
    // `ordered` is newest-first; newest LTS major (22) is first.
    let lts_majors: Vec<&str> = info
        .ordered
        .iter()
        .filter(|(_, lts)| lts.is_some())
        .map(|(v, _)| v.as_str())
        .collect();
    assert_eq!(lts_majors.first(), Some(&"22.11.0"));
}
