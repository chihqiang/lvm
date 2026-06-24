//! LVM - Language Version Manager
//! 多语言版本管理工具，支持通过插件式架构扩展新的语言

mod commands;
mod config;
mod dispatch;

use anyhow::{Result, bail};
use commands::output;

mod plugin;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut registry = plugin::PluginRegistry::new();
    registry.register(Box::new(plugin::node::NodePlugin));
    registry.register(Box::new(plugin::go::GoPlugin));

    let mut cmd = commands::cli::build_cli();
    let cli = cmd.get_matches_mut();

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

            plugin::set_offline(offline);

            let lts_ver = lts.map(|v| {
                if v.is_empty() || v == "*" {
                    "lts/*".to_string()
                } else {
                    format!("lts/{v}")
                }
            });
            let effective_ver_ref: Option<&str> = lts_ver.as_deref().or(arg_ver);

            let plans = dispatch::resolve_install_args(arg_lang, effective_ver_ref, &registry)?;
            let last_plan = plans.last().cloned();
            for (lang, ver) in &plans {
                commands::install(&registry, lang, ver.as_deref(), no_default)?;
            }
            if save && let Some(msg) = dispatch::write_save(&registry, last_plan.as_ref())? {
                output::info(msg);
            }
            if let Some(from_ver) = reinstall_from {
                for (lang, _) in &plans {
                    commands::reinstall_packages(&registry, lang, from_ver)?;
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

            let plans = dispatch::resolve_install_args(arg_lang, arg_ver, &registry)?;
            let last_plan = plans.last().cloned();
            for (lang, ver) in &plans {
                commands::use_version(&registry, lang, ver.as_deref(), set_default)?;
            }
            if save && let Some(msg) = dispatch::write_save(&registry, last_plan.as_ref())? {
                output::info(msg);
            }
            Ok(())
        }
        Some(("list", sub)) => {
            let language = dispatch::req_arg(sub, "language")?;
            commands::list(&registry, language)
        }
        Some(("list-remote", sub)) => {
            let language = dispatch::req_arg(sub, "language")?;
            let lts_only = sub.get_flag("lts");
            commands::list_remote(&registry, language, lts_only)
        }
        Some(("current", sub)) => {
            if let Some(lang) = sub.get_one::<String>("language") {
                commands::current(&registry, lang)
            } else {
                commands::current_all(&registry)
            }
        }
        Some(("which", sub)) => {
            let language = dispatch::req_arg(sub, "language")?;
            let version = sub
                .get_one::<String>("version")
                .map_or("current", String::as_str);
            if version == "current" {
                let plugin = commands::get_plugin(&registry, language)?;
                let Some(cur) = plugin.current_version()? else {
                    bail!("No active version for {language}");
                };
                return commands::which(&registry, language, &cur);
            }
            commands::which(&registry, language, version)
        }
        Some(("alias", sub)) => {
            let language = dispatch::req_arg(sub, "language")?;
            let name = sub.get_one::<String>("name").map(String::as_str);
            let version = sub.get_one::<String>("version").map(String::as_str);
            commands::alias(language, name, version)
        }
        Some(("unalias", sub)) => {
            let language = dispatch::req_arg(sub, "language")?;
            let name = dispatch::req_arg(sub, "name")?;
            commands::unalias(language, name)
        }
        Some(("cache", sub)) => match sub.subcommand() {
            Some(("dir", _)) => {
                println!("{}", config::downloads_dir().display());
                Ok(())
            }
            Some(("clear", _)) => commands::cache_clear(),
            _ => {
                output::info("Usage: lvm cache <dir|clear>");
                Ok(())
            }
        },
        Some(("uninstall", sub)) => {
            let language = dispatch::req_arg(sub, "language")?;
            let version = dispatch::req_arg(sub, "version")?;
            commands::uninstall(&registry, language, version)
        }
        Some(("env", sub)) => {
            if let Some(shell) = sub.get_one::<String>("shell") {
                commands::env_completions(shell);
            } else {
                commands::env();
            }
            Ok(())
        }
        Some(("hook", _)) => {
            commands::hook();
            Ok(())
        }
        Some(("debug", _)) => {
            commands::debug(&registry);
            Ok(())
        }
        None => {
            cmd.print_help().ok();
            println!();
            Ok(())
        }
        Some((name, _)) => {
            bail!("Unknown command {name}")
        }
    }
}
