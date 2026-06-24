use clap_complete::{Shell, generate};

use crate::config;
use crate::plugin;

use super::cli;
use super::output;

pub(crate) fn env() {
    let lvm_home_path = config::lvm_home();
    let bin_path = lvm_home_path.join(config::bin_dir_name());
    let go_packages = lvm_home_path.join("current/go/packages");
    let node_bin = lvm_home_path.join("current/node/bin");

    if cfg!(windows) {
        // cmd.exe / PowerShell 语法
        let sep = plugin::path_separator();
        let lvm_home = lvm_home_path.display();
        let gopath = go_packages.display();
        let go_bin = go_packages.join("bin");
        println!("set \"LVM_HOME={lvm_home}\"");
        println!("set \"GOPATH={gopath}\"");
        println!(
            "set \"PATH={}{sep}{}{sep}{}{sep}%PATH%\"",
            go_bin.display(),
            node_bin.display(),
            bin_path.display(),
        );
    } else {
        // POSIX shell 语法
        println!("export LVM_HOME=\"{}\"", lvm_home_path.display());
        println!("export GOPATH=\"{}\"", go_packages.display());
        let sep = plugin::path_separator();
        println!(
            "export PATH=\"{}{sep}{}{sep}{}{sep}$PATH\"",
            go_packages.join("bin").display(),
            node_bin.display(),
            bin_path.display(),
        );
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
