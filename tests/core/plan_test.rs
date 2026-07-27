use anyhow::Result;
use lvm::core::plan::{resolve_install_args, write_current_versions_to_lvmrc};
use lvm::language::{Language, LanguageRegistry};
use serial_test::serial;

struct DummyLanguage(&'static str);

impl Language for DummyLanguage {
    fn name(&self) -> &str {
        self.0
    }

    fn install(&self, _version: Option<&str>) -> Result<String> {
        Ok("1.0.0".to_string())
    }

    fn latest_version(&self) -> Result<String> {
        Ok("1.0.0".to_string())
    }
}

struct CurrentLanguage {
    name: &'static str,
    current: Option<&'static str>,
}

impl Language for CurrentLanguage {
    fn name(&self) -> &str {
        self.name
    }

    fn install(&self, _version: Option<&str>) -> Result<String> {
        Ok("1.0.0".to_string())
    }

    fn latest_version(&self) -> Result<String> {
        Ok("1.0.0".to_string())
    }

    fn current_version(&self) -> Result<Option<String>> {
        Ok(self.current.map(str::to_string))
    }
}

fn registry(names: &[&'static str]) -> LanguageRegistry {
    let mut registry = LanguageRegistry::new();
    for name in names {
        registry.register(Box::new(DummyLanguage(name)));
    }
    registry
}

#[test]
fn resolve_language_without_version() {
    let registry = registry(&["node", "go"]);
    let plans = resolve_install_args(Some("node"), None, &registry).unwrap();
    assert_eq!(plans, vec![("node".to_string(), None)]);
}

#[test]
fn resolve_language_with_version() {
    let registry = registry(&["node", "go"]);
    let plans = resolve_install_args(Some("node"), Some("22"), &registry).unwrap();
    assert_eq!(plans, vec![("node".to_string(), Some("22".to_string()))]);
}

#[test]
fn resolve_bare_version_when_single_language_registered() {
    let registry = registry(&["node"]);
    let plans = resolve_install_args(Some("22"), None, &registry).unwrap();
    assert_eq!(plans, vec![("node".to_string(), Some("22".to_string()))]);
}

#[test]
fn reject_bare_version_when_multiple_languages_registered() {
    let registry = registry(&["node", "go"]);
    let err = resolve_install_args(Some("22"), None, &registry).unwrap_err();
    assert!(err.to_string().contains("Ambiguous argument '22'"));
}

#[test]
#[serial]
fn resolve_no_args_from_lvmrc() {
    let registry = registry(&["node", "go"]);
    let dir = tempfile::tempdir().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    std::fs::write(".lvmrc", "node=22.0.0\ngo=1.22.0\n").unwrap();

    let mut plans = resolve_install_args(None, None, &registry).unwrap();
    plans.sort();

    std::env::set_current_dir(cwd).unwrap();
    assert_eq!(
        plans,
        vec![
            ("go".to_string(), Some("1.22.0".to_string())),
            ("node".to_string(), Some("22.0.0".to_string()))
        ]
    );
}

#[test]
#[serial]
fn write_current_versions_saves_only_active_languages() {
    let mut registry = LanguageRegistry::new();
    registry.register(Box::new(CurrentLanguage {
        name: "node",
        current: Some("22.0.0"),
    }));
    registry.register(Box::new(CurrentLanguage {
        name: "go",
        current: None,
    }));

    let dir = tempfile::tempdir().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let plans = vec![
        ("node".to_string(), Some("22".to_string())),
        ("go".to_string(), Some("1.22".to_string())),
    ];
    let msg = write_current_versions_to_lvmrc(&registry, &plans).unwrap();
    let content = std::fs::read_to_string(dir.path().join(".lvmrc")).unwrap();

    std::env::set_current_dir(cwd).unwrap();

    assert_eq!(msg, Some("Wrote 1 language(s) to .lvmrc".to_string()));
    assert_eq!(content, "node=22.0.0\n");
}

#[test]
#[serial]
fn write_current_versions_returns_none_when_nothing_active() {
    let mut registry = LanguageRegistry::new();
    registry.register(Box::new(CurrentLanguage {
        name: "node",
        current: None,
    }));

    let dir = tempfile::tempdir().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let plans = vec![("node".to_string(), Some("22".to_string()))];
    let msg = write_current_versions_to_lvmrc(&registry, &plans).unwrap();
    let lvmrc_exists = dir.path().join(".lvmrc").exists();

    std::env::set_current_dir(cwd).unwrap();

    assert_eq!(msg, None);
    assert!(!lvmrc_exists);
}
