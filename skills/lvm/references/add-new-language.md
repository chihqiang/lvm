# 如何新增语言

以 Dart 模块为模板，新增一个语言需要以下步骤：

## 1. 创建目录和三个文件

```
src/language/{name}/
├── config.rs      # 镜像源、OS/arch 映射、URL 构造
├── version.rs     # 版本列表获取、最新版本获取
└── mod.rs         # Language trait 实现 + resolve_version()
```

## 2. `config.rs` — 配置与 URL 生成

```rust
use std::env;
use std::sync::OnceLock;

static MIRROR: OnceLock<String> = OnceLock::new();

pub(crate) fn mirror() -> &'static str {
    MIRROR.get_or_init(|| {
        env::var("LVM_{NAME}_MIRROR")
            .unwrap_or_else(|_| "https://default-mirror.com".to_string())
    })
}

// OS arch 映射（平台 → 上游命名）
fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "arm64",
        "x86_64"  => "x64",
        other => other,
    }
}

pub(crate) fn target_os() -> &'static str { os_name(env::consts::OS) }
pub(crate) fn target_arch() -> &'static str { arch_name(env::consts::ARCH) }

pub(crate) fn tarball_filename(version: &str, os: &str, arch: &str) -> String {
    format!("{name}-{version}-{os}-{arch}.tar.gz")
}

pub(crate) fn download_url(version: &str, os: &str, arch: &str) -> String {
    format!("{}/download/{}-{}-{}.tar.gz", mirror(), version, os, arch)
}
```

### 关键模式

- **镜像源**: 使用 `OnceLock` + `env::var("LVM_{NAME}_MIRROR")`，允许用户覆盖
- **OS/arch 映射**: 将 Rust 标准命名 (`env::consts::OS` = `"macos"`, `env::consts::ARCH` = `"aarch64"`) 转为上游使用的命名
- **版本前缀**: 如果语言的版本带 `v` 前缀（如 Go、Node），`version_prefix()` 返回 `"v"`；否则返回 `""`
- **缓存文件名**: 提供 `{name}-versions.txt` 和 `{name}-latest-version.json`

### 特殊 case：全平台 .zip

如果语言在所有平台都使用 `.zip`（如 Dart、Flutter），硬编码：

```rust
const EXT: &str = "zip";

pub(crate) fn tarball_filename(version: &str, os: &str, _arch: &str) -> String {
    format!("dartsdk-{os}-{arch}-release.{EXT}")
}
```

### 特殊 case：通用二进制（不分 arch）

如果下载 URL 不含 arch（如 Flutter 通用二进制），`_arch` 参数保留但不使用：

```rust
pub(crate) fn tarball_filename(version: &str, os: &str, _arch: &str) -> String {
    format!("flutter_{os}_{version}-stable.zip")
}
```

## 3. `mod.rs` — Language trait 实现

```rust
pub(crate) mod config;
mod version;

use anyhow::{Context, Result, bail};
use super::Language;
use crate::language;

pub struct {Name}Language;

impl Language for {Name}Language {
    fn name(&self) -> &'static str { "{name}" }
    fn version_prefix(&self) -> &'static str { "" }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("{Name}", &resolved);
            return Ok(resolved);
        }

        let os = config::target_os();
        let archs: &[&str] = if config::target_arch() != "x64" {
            &[config::target_arch(), "x64"]
        } else {
            &[config::target_arch()]
        };

        for (i, &arch) in archs.iter().enumerate() {
            if i > 0 && self.is_installed(&version_dir) {
                return Ok(resolved);
            }

            let url = config::download_url(&resolved, os, arch);
            let tar_path = crate::config::downloads_dir_or_default()
                .join(config::tarball_filename(&resolved, os, arch));

            if arch != config::target_arch() {
                language::report_non_native_arch(os, arch);
            }

            match language::download_and_install(
                &url, &tar_path, &resolved, &version_dir,
                "{Name}", |_| Ok(()),
            ) {
                Ok(()) => return Ok(resolved),
                Err(_e) if i + 1 < archs.len() => {
                    language::report_fallback(arch, archs[i + 1]);
                }
                Err(e) => return Err(e),
            }
        }
        bail!("Failed to install {name} {resolved}")
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(crate::config::bin_dir_name())]
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        vec![("{NAME}_HOME", self.current_link())]
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    match version {
        None => {Name}Language::fetch_latest_version(),
        Some(v) => {
            let v = v.trim();
            if v == crate::config::system_version_keyword() {
                bail!("Use 'lvm use system' instead of 'lvm install system'");
            }
            let candidate = v.trim_start_matches('v');
            if let Ok(ver) = semver::Version::parse(candidate) {
                return Ok(ver.to_string());
            }
            let avail: Vec<semver::Version> = {Name}Language::fetch_all_versions()?
                .iter().filter_map(|s| semver::Version::parse(s).ok()).collect();
            language::resolve_partial_version(candidate, &avail, "{Name}")
        }
    }
}
```

### Arch fallback 模式

```rust
let archs: &[&str] = if config::target_arch() != "x64" {
    &[config::target_arch(), "x64"]   // 先试原生，失败后回退 x64
} else {
    &[config::target_arch()]
};
```

- 循环 `archs`，每个 arch 尝试下载+安装
- 成功后 `return Ok(resolved)`
- 失败时 `report_fallback(arch, next_arch)` → 继续下一个 arch
- 有一个特例：Node 的 `resolve_install_version()` 额外支持直接 URL 安装，此时不走 arch fallback

### Zip 格式校验

```rust
let verify_zip = |path: &Path| -> Result<()> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    ZipArchive::new(file).context("Corrupted zip archive")?;
    Ok(())
};
```

## 4. `version.rs` — 版本列表与最新版本

```rust
use anyhow::{Context, Result};
use crate::config;
use crate::language;

impl super::{Name}Language {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| config::default_cache_dir())
            .join("{name}-latest-version.json");
        let text = language::fetch_with_cache(&cache_file, || {
            // 调用 API 获取最新版本
        })?;
        // 解析 text，返回最新版本字符串
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| config::default_cache_dir())
            .join("{name}-versions.txt");
        let text = language::fetch_with_cache(&cache_file, || {
            // 调用 API 获取所有版本列表
        })?;
        // 解析 text，排序，去重，返回 Vec<String>
    }
}
```

### 版本获取策略参考

| 语言 | 最新版本 API | 版本列表 API | 解析方式 |
|------|-------------|-------------|---------|
| Node | `index.json` 取第一项 | `index.json` 全部 | JSON array |
| Go | 从排序后列表取 last | `?mode=json&include=all` | `serde_json::Value` |
| Java | `info/available_releases` 的 `"most_recent_lts"` | 同上 | `serde_json::Value` |
| Python | 从排序后列表取 last | `python.org/ftp/python/` HTML | `<a href="...">` 解析 |
| Dart | `latest/VERSION` JSON 的 `"version"` | S3 XML listing | XML `<Key>` 解析 |
| Flutter | `releases_{os}.json` 的 `current_release.stable` | 同上 JSON | `serde_json::Value` |

## 5. 注册

1. `src/language/mod.rs` — 添加 `pub mod {name};`
2. `src/main.rs` — `registry.register(Box::new(language::{name}::{Name}Language));`
3. 如果语言需要 `.lvmrc` 自动切换，在 `src/commands/use_version/mod.rs` 添加特殊处理
4. 如果语言需要 `.nvmrc` 兼容，参考 `node/nvmrc.rs` 和 `dispatch.rs`
