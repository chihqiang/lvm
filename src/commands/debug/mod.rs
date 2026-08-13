use lvm::core::config;
use lvm::language::LanguageRegistry;

/// Print debug info (system, registry, PATH, etc.)
pub(crate) fn debug(registry: &LanguageRegistry) {
    let version = env!("CARGO_PKG_VERSION");
    let home = config::lvm_home().unwrap_or_else(|_| std::path::PathBuf::from("./.lvm"));
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    println!("lvm v{version}");
    println!("OS:          {os}");
    println!("Arch:        {arch}");
    println!("LVM_HOME:    {}", home.display());
    println!(
        "Downloads:   {}",
        config::downloads_dir().map_or_else(|_| "<error>".to_string(), |p| p.display().to_string())
    );
    println!(
        "Cache:       {}",
        config::cache_dir().map_or_else(|_| "<error>".to_string(), |p| p.display().to_string())
    );
    println!();

    println!("Registered languages:");
    for name in registry.list_names() {
        if let Some(lang) = registry.get(name) {
            let cur = match lang.current_version() {
                Ok(Some(v)) => v,
                Ok(None) => String::new(),
                Err(ref e) => format!("<error: {e}>"),
            };
            println!("  {name}: current={cur}");
        }
    }
    println!();

    // Check PATH for conflicts
    let bin_path = home.join(config::BIN_DIR);
    println!("PATH entries:");
    if let Some(paths) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&paths) {
            let marker = if p == bin_path { " ← lvm" } else { "" };
            let conflicts: Vec<String> = registry
                .list_names()
                .iter()
                .filter_map(|name| {
                    // Account for platform executable suffix (e.g. node.exe).
                    let exe = p.join(format!("{}{}", name, std::env::consts::EXE_SUFFIX));
                    if exe.exists() {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let has_conflict = if conflicts.is_empty() {
                String::new()
            } else {
                format!(" [has {}]", conflicts.join(", "))
            };
            println!("  {}{}{}", p.display(), marker, has_conflict);
        }
    }
}
