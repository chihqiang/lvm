use clap::{Arg, Command, value_parser};

fn language_arg(required: bool) -> Arg {
    Arg::new("language")
        .help("Language name (e.g. node)")
        .required(required)
}

fn version_arg(required: bool, help: &'static str) -> Arg {
    Arg::new("version").help(help).required(required)
}

fn install_subcommand() -> Command {
    Command::new("install")
        .about("Install a language version")
        .arg(language_arg(false))
        .arg(version_arg(false, "Version to install (latest if omitted)"))
        .arg(
            Arg::new("save")
                .long("save")
                .short('w')
                .help("Write version to .lvmrc")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lts")
                .long("lts")
                .help("Install latest LTS version (optional: --lts=<name>)")
                .num_args(0..=1)
                .default_missing_value("*"),
        )
        .arg(
            Arg::new("reinstall-packages-from")
                .long("reinstall-packages-from")
                .help("Reinstall global packages from a version")
                .value_name("VERSION"),
        )
        .arg(
            Arg::new("no-default")
                .long("no-default")
                .help("Do not set as default after install")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("offline")
                .long("offline")
                .help("Install from cache only (no network)")
                .action(clap::ArgAction::SetTrue),
        )
}

fn use_subcommand() -> Command {
    Command::new("use")
        .about("Switch to a specific version")
        .arg(language_arg(false))
        .arg(version_arg(false, "Version to use"))
        .arg(
            Arg::new("no-default")
                .long("no-default")
                .help("Do not set as default")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("save")
                .long("save")
                .short('w')
                .help("Write version to .lvmrc")
                .action(clap::ArgAction::SetTrue),
        )
}

fn list_subcommand() -> Command {
    Command::new("list")
        .about("List installed versions")
        .arg(language_arg(true))
}

fn list_remote_subcommand() -> Command {
    Command::new("list-remote")
        .about("List remote available versions")
        .arg(language_arg(true))
        .arg(
            Arg::new("lts")
                .long("lts")
                .help("Only show LTS versions")
                .action(clap::ArgAction::SetTrue),
        )
}

fn current_subcommand() -> Command {
    Command::new("current")
        .about("Show currently active version")
        .arg(language_arg(false))
}

fn which_subcommand() -> Command {
    Command::new("which")
        .about("Show path to a version binary")
        .arg(language_arg(true))
        .arg(version_arg(
            false,
            "Version to show path for (default: current)",
        ))
}

fn alias_subcommand() -> Command {
    Command::new("alias")
        .about("Manage version aliases")
        .arg(language_arg(true))
        .arg(Arg::new("name").help("Alias name").required(false))
        .arg(version_arg(false, "Version to alias to"))
}

fn unalias_subcommand() -> Command {
    Command::new("unalias")
        .about("Remove a version alias")
        .arg(language_arg(true))
        .arg(Arg::new("name").help("Alias name").required(true))
}

fn cache_subcommand() -> Command {
    Command::new("cache")
        .about("Manage download cache")
        .subcommand(Command::new("dir").about("Show cache directory path"))
        .subcommand(Command::new("clear").about("Clear download cache"))
}

fn uninstall_subcommand() -> Command {
    Command::new("uninstall")
        .about("Uninstall a version")
        .arg(language_arg(true))
        .arg(version_arg(true, "Version to uninstall"))
}

fn prune_subcommand() -> Command {
    Command::new("prune")
        .about("Remove old versions, keeping the most recent N")
        .arg(language_arg(true))
        .arg(
            Arg::new("keep")
                .long("keep")
                .short('n')
                .help("Number of recent versions to keep (default: 3)")
                .value_name("N")
                .value_parser(value_parser!(usize))
                .default_value("3"),
        )
}

fn env_subcommand() -> Command {
    Command::new("env")
        .about("Print shell integration script (environment variables)")
        .arg(
            Arg::new("shell")
                .long("shell")
                .help("Generate completions for a specific shell (bash, zsh, fish)")
                .value_name("SHELL"),
        )
}

fn hook_subcommand() -> Command {
    Command::new("hook")
        .about("Print auto-switch shell hook for .lvmrc")
        .arg(
            Arg::new("shell")
                .long("shell")
                .help("Shell type (bash, zsh, fish, powershell)")
                .value_name("SHELL"),
        )
}

fn debug_subcommand() -> Command {
    Command::new("debug").about("Print debug information for troubleshooting")
}

pub(crate) fn build_cli() -> Command {
    Command::new("lvm")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Language Version Manager")
        .subcommand_required(false)
        .subcommand(install_subcommand())
        .subcommand(use_subcommand())
        .subcommand(list_subcommand())
        .subcommand(list_remote_subcommand())
        .subcommand(current_subcommand())
        .subcommand(which_subcommand())
        .subcommand(alias_subcommand())
        .subcommand(unalias_subcommand())
        .subcommand(cache_subcommand())
        .subcommand(uninstall_subcommand())
        .subcommand(prune_subcommand())
        .subcommand(env_subcommand())
        .subcommand(hook_subcommand())
        .subcommand(debug_subcommand())
}
