# 项目结构

```
src/
├── main.rs              # 入口，注册所有语言，启动 CLI
├── core/                # 核心基础设施（与语言无关）
│   ├── mod.rs           # 重导出全部 pub(crate) 符号
│   ├── config.rs        # 路径、超时、颜色、别名、.lvmrc 等
│   ├── download.rs      # 下载 + 缓存 + 安装流程
│   ├── extract.rs       # 解压 .tar.gz / .zip（自动 strip 顶层目录）
│   ├── http.rs          # HTTP 请求（ureq）
│   ├── checksum.rs      # SHA256 校验
│   ├── fslink.rs        # 符号链接管理、archive_ext、exe_suffix
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
    └── flutter/         # Flutter SDK 实现
```

## 核心基础设施（`src/core/`）

每个模块负责单一职责，语言无关：

- **`config.rs`**: 路径（`lvm_home`, `bin_dir_name`, `current_dir_name`）、超时、颜色、别名读写、`.lvmrc` 解析/写入
- **`download.rs`**: 下载进度条、断点续传、离线模式、`download_and_install`（下载→解压→重命名→清理）
- **`extract.rs`**: 自动检测 zip/tar.gz，`strip_top_level` 去掉 archive 顶层目录
- **`http.rs`**: 基于 ureq 的 HTTP 请求封装、离线模式开关
- **`checksum.rs`**: SHA256 文件校验
- **`fslink.rs`**: 原子替换式 symlink 管理、`archive_ext`/`exe_suffix`/`path_separator`
- **`report.rs`**: 全局缓冲 Vec + 延迟 flush，预定义报告函数
- **`version.rs`**: 版本比较排序、模糊匹配（`resolve_partial_version`）

## CLI 命令（`src/commands/`）

| 命令 | 文件 | 说明 |
|------|------|------|
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

## 语言模块（`src/language/`）

每个语言三个文件：

```
{name}/
├── config.rs   — 镜像源 ENV、OS/arch 映射、URL 构造、缓存文件名
├── version.rs  — fetch_latest_version()、fetch_all_versions()
└── mod.rs      — Language trait 实现 + resolve_version()
```
