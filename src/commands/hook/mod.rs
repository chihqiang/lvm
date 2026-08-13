use lvm::core::config;

use crate::commands::output;

const NVM_FILENAME: &str = ".nvmrc";

fn lvm_auto_function() -> String {
    std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("lvm"))
        .to_string_lossy()
        .to_string()
}

/// Shared shell function body for bash/zsh auto-switch.
/// Prints the `__lvm_auto` bash/zsh compatible function definition.
fn print_lvm_auto_sh() {
    let lvm_bin = lvm_auto_function();
    let lvmrc = config::LVM_FILENAME;
    println!(
        "__lvm_auto() {{ local dir=\"$PWD\" parent; while [[ -n \"$dir\" ]]; do [[ -f \"$dir/{lvmrc}\" || -f \"$dir/{NVM_FILENAME}\" ]] && {{ command -v \"{lvm_bin}\" &>/dev/null && \"{lvm_bin}\" use --no-default --skip-install 2>/dev/null || true; return; }}; parent=\"$(dirname \"$dir\")\"; [[ \"$parent\" == \"$dir\" ]] && return; dir=\"$parent\"; done; }}"
    );
}

fn hook_bash() {
    print_lvm_auto_sh();
    println!(
        "__lvm_auto; [[ \"${{PROMPT_COMMAND-}}\" != *__lvm_auto* ]] && PROMPT_COMMAND=\"__lvm_auto;${{PROMPT_COMMAND-}}\" || true"
    );
}

fn hook_zsh() {
    print_lvm_auto_sh();
    println!("autoload -Uz add-zsh-hook && add-zsh-hook chpwd __lvm_auto && __lvm_auto");
}

fn hook_fish() {
    let lvmrc = config::LVM_FILENAME;
    println!("function __lvm_auto --on-variable PWD --description \"Auto-switch .lvmrc versions\"");
    println!("    set dir $PWD");
    println!("    while test -n \"$dir\"");
    println!("        if test -f \"$dir/{lvmrc}\"; or test -f \"$dir/{NVM_FILENAME}\"");
    println!("            if command -q lvm");
    println!("                lvm use --no-default --skip-install 2>/dev/null");
    println!("            end");
    println!("            return");
    println!("        end");
    println!("        set parent (dirname \"$dir\")");
    println!("        if test \"$parent\" = \"$dir\"");
    println!("            return");
    println!("        end");
    println!("        set dir $parent");
    println!("    end");
    println!("end");
    println!("__lvm_auto");
}

fn hook_powershell() {
    let lvmrc = config::LVM_FILENAME;
    println!("$__lvm_original_prompt = $function:prompt");
    println!("function global:prompt {{");
    println!("    $dir = Get-Location");
    println!("    while ($null -ne $dir) {{");
    println!(
        "        if ((Test-Path (Join-Path $dir '{lvmrc}')) -or (Test-Path (Join-Path $dir '{NVM_FILENAME}'))) {{"
    );
    println!("            $null = & lvm use --no-default --skip-install 2>&1");
    println!("            break");
    println!("        }}");
    println!("        $dir = $dir.Parent");
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
            print_lvm_auto_sh();
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
