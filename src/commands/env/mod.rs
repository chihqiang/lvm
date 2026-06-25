use clap_complete::{Shell, generate};

use crate::config;
use crate::language::{self, LanguageRegistry};

use crate::commands::{cli, output};

pub(crate) fn env(registry: &LanguageRegistry) {
    let lvm_home_path = config::lvm_home();
    let bin_path = lvm_home_path.join(config::bin_dir_name());

    let mut path_entries = Vec::new();
    let mut extra_vars = Vec::new();
    for name in registry.list_names() {
        if let Some(lang) = registry.get(name) {
            path_entries.push(lang.current_link().join(config::bin_dir_name()));
            path_entries.extend(lang.env_extra_paths());
            extra_vars.extend(lang.env_extra_vars());
        }
    }
    path_entries.push(bin_path);

    let sep = language::path_separator();
    let path_str = path_entries
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(sep);

    if cfg!(windows) {
        println!("set \"LVM_HOME={}\"", lvm_home_path.display());
        for (key, val) in extra_vars {
            println!("set \"{key}={}\"", val.display());
        }
        println!("set \"PATH={path_str};%PATH%\"");
    } else {
        println!("export LVM_HOME=\"{}\"", lvm_home_path.display());
        for (key, val) in extra_vars {
            println!("export {key}=\"{}\"", val.display());
        }
        println!("export PATH=\"{path_str}:$PATH\"");
    }
}

pub(crate) fn env_completions(shell: &str) {
    let mut cmd = cli::build_cli();
    match shell {
        "bash" => generate(Shell::Bash, &mut cmd, "lvm", &mut std::io::stdout()),
        "zsh" => generate(Shell::Zsh, &mut cmd, "lvm", &mut std::io::stdout()),
        "fish" => generate(Shell::Fish, &mut cmd, "lvm", &mut std::io::stdout()),
        _ => output::warn(format!(
            "Unknown shell '{shell}', supported: bash, zsh, fish"
        )),
    }
}
