# 项目结构

```text
src/
├── main.rs              # 入口，注册所有语言，启动 CLI
├── core/                # 核心基础设施（与语言无关）
│   ├── mod.rs           # 重导出全部 pub(crate) 符号
│   ├── alias.rs         # 别名 CRUD、default 版本
│   ├── checksum.rs      # SHA256 校验
│   ├── config.rs        # 基础常量（路径名、超时、关键字）+ 路径函数
│   ├── display.rs       # 颜色代码、LTS 标记、勾号
│   ├── extract.rs       # 解压 .tar.gz / .zip（自动 strip 顶层目录）
│   ├── fslink.rs        # 符号链接管理、archive_ext、exe_suffix
│   ├── http.rs          # HTTP 请求 + 下载 + 缓存 + 安装流程
│   ├── lvmrc.rs         # .lvmrc 读写、向上遍历查找
│   ├── report.rs        # 报告消息缓冲系统（先收集后 flush）
│   └── version.rs       # 版本排序、模糊匹配
├── commands/            # CLI 子命令实现
│   ├── cli.rs           # 参数定义（clap）
│   ├── dispatch.rs      # 路由调度 + .lvmrc/.nvmrc 解析
│   ├── install/mod.rs   # install 命令
│   ├── use_version/     # use 命令 + 版本解析链
│   └── ...              # 其他子命令
└── language/            # 各语言实现
    ├── mod.rs           # 重导出 core 符号 + pub mod 声明
    ├── language_trait.rs # Language trait 定义
    ├── registry.rs      # 语言注册表
    ├── node/            # Node.js 实现（含 .nvmrc 支持）
    ├── go/              # Go 实现
    ├── java/            # Java（Adoptium API）实现
    ├── python/          # Python（python-build-standalone）实现
    ├── dart/            # Dart SDK 实现
    ├── flutter/         # Flutter SDK 实现
    ├── kotlin/          # Kotlin 编译器实现
    └── rust/            # Rust 工具链实现
```

## 核心基础设施（`src/core/`）

每个模块负责单一职责，语言无关：

- **`alias.rs`**: 别名 CRUD（`set_alias`、`get_alias`、`remove_alias`）、`set_default_version` / `get_default_version`
- **`config.rs`**: 基础常量（`BIN_DIR`、`CURRENT_DIR`、`SYSTEM_VERSION_KEYWORD`、`LTS_PREFIX`、超时等）和路径函数（`lvm_home`、`downloads_dir`、`cache_path` 等）
- **`display.rs`**: 颜色代码常量、`LTS_MARKER`、`INSTALLED_CHECK_MARK`、`colored_check_mark()`、`use_color()`
- **`extract.rs`**: 自动检测 zip/tar.gz，`strip_top_level` 去掉 archive 顶层目录
- **`http.rs`**: 基于 ureq 的 HTTP 请求封装 + 下载进度条、断点续传、离线模式、缓存 + 安装流程（`download`、`download_and_install`、`fetch_with_cache`、`fetch_from_mirror`）
- **`lvmrc.rs`**: `.lvmrc` 解析/写入、`find_rc_file` 向上遍历查找
- **`checksum.rs`**: SHA256 文件校验
- **`fslink.rs`**: 原子替换式 symlink 管理、`archive_ext`/`exe_suffix`/`path_separator`
- **`report.rs`**: 全局缓冲 Vec + 延迟 flush，预定义报告函数
- **`version.rs`**: 版本比较排序、模糊匹配（`resolve_partial_version`）

## CLI 命令（`src/commands/`）

| 命令 | 文件 | 说明 |
| ------ | ------ | ------ |
| `install` | `install/mod.rs` | 安装并切换 |
| `use` | `use_version/mod.rs` | 切换版本 |
| `list` | `list/mod.rs` | 已安装列表 |
| `list-remote` | `list/remote.rs` | 远程版本列表 |
| `current` | `current/mod.rs` | 当前版本 |
| `which` | `which/mod.rs` | 二进制路径 |
| `alias` | `alias/mod.rs` | 别名管理 |
| `unalias` | `alias/unalias.rs` | 删除别名 |
| `uninstall` | `uninstall/mod.rs` | 卸载 |
| `prune` | `prune/mod.rs` | 清理旧版本 |
| `cache` | `cache/mod.rs` | 缓存管理 |
| `env` | `env/mod.rs` | 环境变量输出 |
| `hook` | `hook/mod.rs` | Shell 自动切换 |
| `debug` | `debug/mod.rs` | 调试诊断 |
| `reinstall` | `reinstall/mod.rs` | 重装全局包 |
| `output.rs` | — | 非缓冲输出（`info`/`warn`） |

## 语言模块（`src/language/`）

每个语言至少三个文件：

```text
{name}/
├── config.rs   — 镜像源 ENV、OS/arch 映射、URL 构造、缓存文件名
├── version.rs  — fetch_latest_version()、fetch_all_versions()
└── mod.rs      — Language trait 实现 + resolve_version()
```

Node.js 有额外的 `nvmrc.rs`（`.nvmrc` 文件读取）和 `lts.rs`（LTS 版本解析）。
