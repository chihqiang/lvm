use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use semver::VersionReq;

use crate::config;
use crate::plugin::PluginRegistry;

/// 判断字符串是否像版本号（数字点号格式或 semver 范围表达式）
pub(crate) fn is_version_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let stripped = s.strip_prefix('v').unwrap_or(s);
    if !stripped.is_empty()
        && !stripped.starts_with('.')
        && !stripped.ends_with('.')
        && !stripped.contains("..")
        && stripped.chars().any(|c| c.is_ascii_digit())
        && stripped.chars().all(|c| c.is_ascii_digit() || c == '.')
    {
        return true;
    }
    VersionReq::parse(s).is_ok()
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
    registry: &PluginRegistry,
) -> Result<Vec<(String, Option<String>)>> {
    match (arg_lang, arg_ver) {
        (Some(lang), None) if is_version_like(lang) => {
            let names = registry.list_names();
            if names.len() == 1 {
                Ok(vec![(names[0].to_string(), Some(lang.to_string()))])
            } else if names.is_empty() {
                bail!("No plugins registered, cannot resolve version: {lang}")
            } else {
                Ok(vec![(lang.to_string(), None)])
            }
        }
        (Some(lang), None) => Ok(vec![(lang.to_string(), None)]),
        (Some(lang), Some(ver)) => Ok(vec![(lang.to_string(), Some(ver.to_string()))]),
        (None, _) => {
            let path = config::lvmrc::find_lvmrc().context("No .lvmrc file found")?;
            let map = config::lvmrc::parse_lvmrc(&path)?;
            let vec: Vec<_> = map.into_iter().map(|(k, v)| (k, Some(v))).collect();
            if vec.is_empty() {
                bail!("No .lvmrc file found");
            }
            Ok(vec)
        }
    }
}

/// 将当前版本写入 .lvmrc，返回要打印的消息
pub(crate) fn write_save(
    registry: &PluginRegistry,
    plan: Option<&(String, Option<String>)>,
) -> Result<Option<String>> {
    if let Some((lang, _)) = plan
        && let Some(plugin) = registry.get(lang)
        && let Some(cur) = plugin.current_version()?
    {
        config::lvmrc::write_lvmrc(lang, &cur)?;
        return Ok(Some(format!("Wrote {lang}={cur} to .lvmrc")));
    }
    Ok(None)
}
