use anyhow::{Context, Result, bail};
use clap::{ArgMatches, Command};
use semver::VersionReq;

use crate::commands;
use crate::commands::output;
use crate::config;
use crate::language;
use crate::language::LanguageRegistry;

pub(crate) fn is_version_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if VersionReq::parse(s).is_ok() {
        return true;
    }
    let stripped = s.trim_start_matches('v');
    !stripped.is_empty()
        && !stripped.starts_with('.')
        && !stripped.ends_with('.')
        && stripped.chars().any(|c| c.is_ascii_digit())
        && stripped.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// 从 clap 匹配中提取必需参数
pub(crate) fn req_arg<'a>(sub: &'a ArgMatches, name: &str) -> Result<&'a str> {
    sub.get_one::<String>(name)
        .map(String::as_str)
        .with_context(|| format!("Missing required argument: {name}"))
}

/// 解析 install/use 的 language 和 version 参数
pub(crate) fn resolve_install_args(
    arg_lang: Option<&str>,
    arg_ver: Option<&str>,
    registry: &LanguageRegistry,
) -> Result<Vec<(String, Option<String>)>> {
    match (arg_lang, arg_ver) {
        (Some(lang), None) if is_version_like(lang) => {
            let names = registry.list_names();
            if names.len() == 1 {
                Ok(vec![(names[0].to_string(), Some(lang.to_string()))])
            } else if names.is_empty() {
                bail!("No languages registered, cannot resolve version: {lang}")
            } else {
                let available = names.join(", ");
                bail!(
                    "Ambiguous argument '{lang}': looks like a version, but multiple languages ({available}) are registered. Use 'lvm install <language> {lang}'"
                )
            }
        }
        (Some(lang), None) => Ok(vec![(lang.to_string(), None)]),
        (Some(lang), Some(ver)) => Ok(vec![(lang.to_string(), Some(ver.to_string()))]),
        (None, _) => {
            if let Some(path) = config::find_lvmrc() {
                let map = config::parse_lvmrc(&path)?;
                let vec: Vec<_> = map.into_iter().map(|(k, v)| (k, Some(v))).collect();
                if vec.is_empty() {
                    bail!(".lvmrc exists but contains no language-version mappings");
                }
                return Ok(vec);
            }
            if let Some(ver) = crate::language::node::read_nvmrc()?
                && !ver.is_empty()
            {
                return Ok(vec![("node".to_string(), Some(ver))]);
            }
            bail!("No .lvmrc or .nvmrc found. Create one or specify arguments")
        }
    }
}

/// 执行命令调度
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
                        return Ok(());
                    }
                    return Err(e);
                }
            };
            let last_plan = plans.last().cloned();
            for (lang, ver) in &plans {
                commands::install(registry, lang, ver.as_deref(), no_default)?;
            }
            if save && let Some(msg) = write_save(registry, last_plan.as_ref())? {
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
            let last_plan = plans.last().cloned();
            for (lang, ver) in &plans {
                commands::use_version(registry, lang, ver.as_deref(), set_default)?;
            }
            if save && let Some(msg) = write_save(registry, last_plan.as_ref())? {
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
                    Ok(d) => println!("{}", d.display()),
                    Err(e) => output::warn(format!("Cannot determine downloads directory: {e}")),
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
            let keep = sub.get_one::<usize>("keep").copied().unwrap_or(3);
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

/// 将当前版本写入 .lvmrc，返回要打印的消息
pub(crate) fn write_save(
    registry: &LanguageRegistry,
    plan: Option<&(String, Option<String>)>,
) -> Result<Option<String>> {
    if let Some((lang, _)) = plan
        && let Some(language) = registry.get(lang)
        && let Some(cur) = language.current_version()?
    {
        config::write_lvmrc(lang, &cur)?;
        return Ok(Some(format!("Wrote {lang}={cur} to .lvmrc")));
    }
    Ok(None)
}
