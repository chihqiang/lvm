use clap::{Arg, Command};

pub(crate) fn build_cli() -> Command {
    Command::new("lvm")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Language Version Manager")
        .subcommand_required(false)
        .subcommand(
            Command::new("install")
                .about("Install a language version")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(false),
                )
                .arg(
                    Arg::new("version")
                        .help("Version to install (latest if omitted)")
                        .required(false),
                )
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
                ),
        )
        .subcommand(
            Command::new("use")
                .about("Switch to a specific version")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(false),
                )
                .arg(Arg::new("version").help("Version to use").required(false))
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
                ),
        )
        .subcommand(
            Command::new("list").about("List installed versions").arg(
                Arg::new("language")
                    .help("Language name (e.g. node)")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("list-remote")
                .about("List remote available versions")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(true),
                )
                .arg(
                    Arg::new("lts")
                        .long("lts")
                        .help("Only show LTS versions")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("current")
                .about("Show currently active version")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("which")
                .about("Show path to a version binary")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(true),
                )
                .arg(
                    Arg::new("version")
                        .help("Version to show path for (default: current)")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("alias")
                .about("Manage version aliases")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(true),
                )
                .arg(Arg::new("name").help("Alias name").required(false))
                .arg(
                    Arg::new("version")
                        .help("Version to alias to")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("unalias")
                .about("Remove a version alias")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(true),
                )
                .arg(Arg::new("name").help("Alias name").required(true)),
        )
        .subcommand(
            Command::new("cache")
                .about("Manage download cache")
                .subcommand(Command::new("dir").about("Show cache directory path"))
                .subcommand(Command::new("clear").about("Clear download cache")),
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall a version")
                .arg(
                    Arg::new("language")
                        .help("Language name (e.g. node)")
                        .required(true),
                )
                .arg(
                    Arg::new("version")
                        .help("Version to uninstall")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("env")
                .about("Print shell integration script (environment variables)")
                .arg(
                    Arg::new("shell")
                        .long("shell")
                        .help("Generate completions for a specific shell (bash, zsh, fish)")
                        .value_name("SHELL"),
                ),
        )
        .subcommand(Command::new("hook").about("Print auto-switch shell hook for .lvmrc"))
        .subcommand(Command::new("debug").about("Print debug information for troubleshooting"))
}
