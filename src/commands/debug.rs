use crate::config;
use crate::plugin::PluginRegistry;

/// 打印调试信息（系统、注册表、PATH 等）
pub(crate) fn debug(registry: &PluginRegistry) {
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
        if let Some(plugin) = registry.get(name) {
            let cur = plugin
                .current_version()
                .ok()
                .flatten()
                .unwrap_or_default();
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
            let node_bin = p.join(crate::plugin::node::node_binary_name());
            let has_node = if node_bin.exists() { " [has node]" } else { "" };
            println!("  {}{}{}", p.display(), marker, has_node);
        }
    }
}
