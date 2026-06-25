use crate::config;
use crate::language::LanguageRegistry;

/// 打印调试信息（系统、注册表、PATH 等）
pub(crate) fn debug(registry: &LanguageRegistry) {
    let version = env!("CARGO_PKG_VERSION");
    let home = config::lvm_home();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    println!("lvm v{version}");
    println!("OS:          {os}");
    println!("Arch:        {arch}");
    println!("LVM_HOME:    {}", home.display());
    println!("Downloads:   {}", config::downloads_dir().display());
    println!("Cache:       {}", config::cache_dir().display());
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
    let bin_path = home.join(config::bin_dir_name());
    println!("PATH entries:");
    if let Some(paths) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&paths) {
            let marker = if p == bin_path { " ← lvm" } else { "" };
            let conflicts: Vec<String> = registry
                .list_names()
                .iter()
                .filter_map(|name| {
                    let exe = p.join(name);
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
