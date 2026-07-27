use anyhow::{Context, Result, bail};
use clap::{ArgMatches, Command};

use crate::commands;
use crate::commands::output;
use lvm::core::config;
use lvm::core::plan::{resolve_install_args, write_current_versions_to_lvmrc};
use lvm::language;
use lvm::language::LanguageRegistry;

/// Extract a required argument from clap matches
fn req_arg<'a>(sub: &'a ArgMatches, name: &str) -> Result<&'a str> {
    sub.get_one::<String>(name)
        .map(String::as_str)
        .with_context(|| format!("Missing required argument: {name}"))
}

/// Execute command dispatch
pub(crate) fn execute(
    cmd: &mut Command,
    cli: &ArgMatches,
    registry: &LanguageRegistry,
) -> Result<()> {
    match cli.subcommand() {
        Some(("install", sub)) => {
            let arg_lang = sub.get_one::<String>("language").map(String::as_str);
            let arg_ver = sub.get_one::<String>("version").map(String::as_str);

            let save = sub.get_flag("save");
            let lts = sub.get_one::<String>("lts").map(String::as_str);
            let offline = sub.get_flag("offline");
            let no_default = sub.get_flag("no-default");
            let reinstall_from = sub
                .get_one::<String>("reinstall-packages-from")
                .map(String::as_str);

            language::set_offline(offline);

            let lts_ver = lts.map(|v| {
                if v.is_empty() || v == "*" {
                    "lts/*".to_string()
                } else {
                    format!("lts/{v}")
                }
            });
            let effective_ver_ref: Option<&str> = lts_ver.as_deref().or(arg_ver);

            let plans = match resolve_install_args(arg_lang, effective_ver_ref, registry) {
                Ok(p) => p,
                Err(e) => {
                    if arg_lang.is_none() {
                        let mut help = crate::commands::cli::install_subcommand();
                        let _ = help.print_help();
                        println!();
                    }
                    return Err(e);
                }
            };
            commands::install_plans(registry, &plans, no_default)?;
            if save && let Some(msg) = write_current_versions_to_lvmrc(registry, &plans)? {
                output::info(msg);
            }
            if let Some(from_ver) = reinstall_from {
                for (lang, _) in &plans {
                    commands::reinstall_packages(registry, lang, from_ver)?;
                }
            }
            Ok(())
        }
        Some(("use", sub)) => {
            let arg_lang = sub.get_one::<String>("language").map(String::as_str);
            let arg_ver = sub.get_one::<String>("version").map(String::as_str);
            let no_default = sub.get_flag("no-default");
            let set_default = !no_default;
            let save = sub.get_flag("save");

            let plans = resolve_install_args(arg_lang, arg_ver, registry)?;
            for (lang, ver) in &plans {
                commands::use_version(registry, lang, ver.as_deref(), set_default)?;
            }
            if save && let Some(msg) = write_current_versions_to_lvmrc(registry, &plans)? {
                output::info(msg);
            }
            Ok(())
        }
        Some(("list", sub)) => {
            let language = req_arg(sub, "language")?;
            commands::list(registry, language)
        }
        Some(("list-remote", sub)) => {
            let language = req_arg(sub, "language")?;
            let lts_only = sub.get_flag("lts");
            commands::list_remote(registry, language, lts_only)
        }
        Some(("current", sub)) => {
            if let Some(lang) = sub.get_one::<String>("language") {
                commands::current(registry, lang)
            } else {
                commands::current_all(registry)
            }
        }
        Some(("which", sub)) => {
            let language = req_arg(sub, "language")?;
            let version = sub
                .get_one::<String>("version")
                .map_or("current", String::as_str);
            commands::which(registry, language, version)
        }
        Some(("alias", sub)) => {
            let language = req_arg(sub, "language")?;
            let name = sub.get_one::<String>("name").map(String::as_str);
            let version = sub.get_one::<String>("version").map(String::as_str);
            commands::alias(language, name, version)
        }
        Some(("unalias", sub)) => {
            let language = req_arg(sub, "language")?;
            let name = req_arg(sub, "name")?;
            commands::unalias(language, name)
        }
        Some(("cache", sub)) => match sub.subcommand() {
            Some(("dir", _)) => {
                match config::downloads_dir() {
                    Ok(d) => println!("Downloads:  {}", d.display()),
                    Err(e) => output::warn(format!("Cannot determine downloads directory: {e}")),
                }
                match config::cache_dir() {
                    Ok(d) => println!("Cache:      {}", d.display()),
                    Err(e) => output::warn(format!("Cannot determine cache directory: {e}")),
                }
                Ok(())
            }
            Some(("clear", _)) => commands::cache_clear(),
            _ => {
                output::info("Usage: lvm cache <dir|clear>");
                Ok(())
            }
        },
        Some(("uninstall", sub)) => {
            let language = req_arg(sub, "language")?;
            let version = req_arg(sub, "version")?;
            commands::uninstall(registry, language, version)
        }
        Some(("prune", sub)) => {
            let language = req_arg(sub, "language")?;
            let keep = *sub
                .get_one::<usize>("keep")
                .expect("keep has default value");
            commands::prune(registry, language, keep)
        }
        Some(("env", sub)) => {
            if let Some(shell) = sub.get_one::<String>("shell") {
                commands::env_completions(shell);
            } else {
                commands::env(registry);
            }
            Ok(())
        }
        Some(("hook", sub)) => {
            commands::hook(sub.get_one::<String>("shell").map(String::as_str));
            Ok(())
        }
        Some(("debug", _)) => {
            commands::debug(registry);
            Ok(())
        }
        None => {
            let _ = cmd.print_help();
            println!();
            Ok(())
        }
        Some((name, _)) => {
            bail!("Unknown command {name}")
        }
    }
}
