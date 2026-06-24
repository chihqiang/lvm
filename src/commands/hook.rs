use crate::config;

pub(crate) fn hook() {
    if cfg!(windows) {
        eprintln!("Shell auto-hook is not supported on Windows");
        eprintln!(
            "Manually add {} to your PATH and use 'lvm use' in your project directories",
            config::lvm_home().join(config::bin_dir_name()).display()
        );
        return;
    }

    let lvm_home_path = config::lvm_home();
    let lvm_bin = lvm_home_path.join(config::bin_dir_name()).join("lvm");
    let lvm_bin = lvm_bin.to_string_lossy();
    let lvmrc = config::lvmrc_filename();

    println!(
        "__lvm_auto() {{ [[ -f {lvmrc} ]] && command -v \"{lvm_bin}\" &>/dev/null && \"{lvm_bin}\" use 2>/dev/null || true; }}"
    );
    println!(
        "[[ -n \"${{BASH_VERSION-}}\" ]] && {{ __lvm_auto; [[ \"${{PROMPT_COMMAND-}}\" != *__lvm_auto* ]] && PROMPT_COMMAND=\"__lvm_auto;${{PROMPT_COMMAND-}}\" || true; }}"
    );
    println!(
        "[[ -n \"${{ZSH_VERSION-}}\" ]] && {{ autoload -Uz add-zsh-hook && add-zsh-hook chpwd __lvm_auto && __lvm_auto; }}"
    );
}
