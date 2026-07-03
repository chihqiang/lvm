use lvm::core::config;

use crate::commands::output;

fn lvm_auto_function() -> (String, String) {
    let lvm_bin = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("lvm"))
        .to_string_lossy()
        .to_string();
    let lvmrc = config::LVM_FILENAME.to_string();
    (lvm_bin, lvmrc)
}

fn hook_bash() {
    let (lvm_bin, lvmrc) = lvm_auto_function();
    println!(
        "__lvm_auto() {{ [[ -f {lvmrc} ]] && command -v \"{lvm_bin}\" &>/dev/null && \"{lvm_bin}\" use 2>/dev/null || true; }}"
    );
    println!(
        "__lvm_auto; [[ \"${{PROMPT_COMMAND-}}\" != *__lvm_auto* ]] && PROMPT_COMMAND=\"__lvm_auto;${{PROMPT_COMMAND-}}\" || true"
    );
}

fn hook_zsh() {
    let (lvm_bin, lvmrc) = lvm_auto_function();
    println!(
        "__lvm_auto() {{ [[ -f {lvmrc} ]] && command -v \"{lvm_bin}\" &>/dev/null && \"{lvm_bin}\" use 2>/dev/null || true; }}"
    );
    println!("autoload -Uz add-zsh-hook && add-zsh-hook chpwd __lvm_auto && __lvm_auto");
}

fn hook_fish() {
    let lvmrc = config::LVM_FILENAME;

    println!("function __lvm_auto --on-variable PWD --description \"Auto-switch .lvmrc versions\"");
    println!("    if test -f {lvmrc}");
    println!("        if command -q lvm");
    println!("            lvm use 2>/dev/null");
    println!("        end");
    println!("    end");
    println!("end");
    println!("__lvm_auto");
}

fn hook_powershell() {
    println!("$__lvm_original_prompt = $function:prompt");
    println!("function global:prompt {{");
    println!("    if (Test-Path .lvmrc) {{");
    println!("        $null = & lvm use 2>&1");
    println!("    }}");
    println!("    & $__lvm_original_prompt");
    println!("}}");
}

pub(crate) fn hook(shell: Option<&str>) {
    if cfg!(windows) {
        if let Some("powershell") = shell {
            hook_powershell();
        } else {
            output::warn("Shell auto-hook is not supported on Windows");
            let home_path =
                config::lvm_home().unwrap_or_else(|_| std::path::PathBuf::from("./.lvm"));
            output::info(format!(
                "Manually add {} to your PATH and use 'lvm use' in your project directories",
                home_path.join(config::BIN_DIR).display()
            ));
        }
        return;
    }

    match shell {
        Some("bash") => hook_bash(),
        Some("zsh") => hook_zsh(),
        Some("fish") => hook_fish(),
        Some("powershell") => hook_powershell(),
        None => {
            let (lvm_bin, lvmrc) = lvm_auto_function();

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
        Some(s) => output::warn(format!(
            "Unknown shell '{s}', supported: bash, zsh, fish, powershell"
        )),
    }
}
