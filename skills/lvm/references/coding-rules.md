# 代码规范

## 配置集中原则

- **不要硬编码语言特定值** — 每个语言的镜像源、OS/arch 映射、URL 格式、缓存文件名放在语言模块的 `config.rs` 中
- **公共配置集中管理** — 所有跨语言共享的常量（路径名 `"bin"`、`"current"`、`"system"` 关键字、`"lts/"` 前缀、超时、颜色、`.lvmrc` 文件名等）统一放在 `src/core/config.rs` 的函数中
- **公共报告消息集中管理** — 通用报告（`report_already_installed`、`report_fallback` 等）放在 `src/core/report.rs`，语言特有的消息在语言模块内用 `report(format!(...))` 直接输出
- **禁止在语言模块中直接写 `"bin"`、`"current"`、`"system"`、`"lts/"` 等字符串字面量** — 通过 `crate::config::bin_dir_name()`、`config::system_version_keyword()` 等函数引用

## 代码风格

- **适当写注释** — 关键逻辑、复杂算法、公共 API 应有中文注释说明意图
- **import 顺序**: 标准库 → 第三方 crate → `crate::` 内部模块，每组空行分隔
- **错误处理**: 使用 `anyhow::{Context, Result, bail}`，需要额外上下文时用 `.with_context(|| format!(...))`，简单错误用 `.context("...")`
- **命名**: Rust 标准命名法 — struct 用 PascalCase，函数/变量用 snake_case，常量用 SCREAMING_SNAKE_CASE
- **不要用 derive** — 不使用 `#[derive(Deserialize)]`，只使用 `serde_json::Value` 手写解析
- **代码不要过于抽象** — 各语言模块可以有一定重复，优先可读性
- **path 字面量集中管理**: 所有 `"bin"`, `"current"`, `"system"`, `"lts/"` 等字符串必须使用 `core/config.rs` 中的函数
- **report 消息集中管理**: 所有通用报告消息使用 `core/report.rs` 中的预定义函数

## CLI 规范

- 所有用户可见输出用 `report()` / `report(format!(...))` 缓冲
- 关键位置调用 `language::flush_reports_to_stdout()` 一次性输出
- 不要直接用 `println!` 或 `eprintln!`（除 `cli.rs` 中的 `print_help`）
- `clap` 子命令定义在 `commands/cli.rs`，实现放在 `commands/{name}/mod.rs`

## Git 规范

- **不要修改 git config**（不要 `git config`、`git commit --no-verify`、`git commit --no-gpg-sign`、`git push --force`）
- **不要 `git add -A`** — 只 stage 需要提交的文件
- commit message 格式: `类型: 简短描述`（如 `feat:`, `fix:`, `chore:`, `refactor:`），用中文
- 不要 amend 已推送的 commit，创建新 commit
- push 之前检查 `git status`、`git diff`、`git log --oneline -10`

## PR 规范

- 创建 PR 时：先 `git status` → `git diff` → `git log --oneline -10` → 检查 remote tracking
- 使用 `gh` CLI 创建 PR，返回 PR URL
- review 所有 commits（不仅是最后一个）
- **不要创建空 commit**

## CI 检查（必须通过）

```bash
cargo fmt --all --check              # 格式
cargo clippy --all-targets -- -D warnings  # lint（无警告）
cargo build --all-targets            # 编译
cargo test --all-targets             # 测试
```

## 跨平台注意事项

- `env::consts::OS` = `"macos"` / `"linux"` / `"windows"`
- `env::consts::ARCH` = `"aarch64"` / `"x86_64"` / `"x86"`
- `std::env::consts::EXE_SUFFIX` = `""`（非Win）/ `".exe"`（Win）
- 语言模块的 `config.rs` 负责将 Rust 标准命名映射到上游命名
- arch fallback 条件统一使用: `if config::target_arch() != "x64"`
- **不写平台特定代码** — arch fallback 用纯 arch 检查
- **不假定 x64 是回退目标** — 所有语言的 arm64 fallback 统一为 x64/amd64/x86_64
- `archive_ext()` Windows 返回 `"zip"`，其他返回 `"tar.gz"`；Dart/Flutter 在所有平台硬编码 `.zip`

## 新增文件注意事项

- 创建新 `.rs` 文件后，必须在其父模块的 `mod.rs` 中添加 `mod` 声明
- 语言模块内部: `config.rs` 用 `pub(crate) mod config;`，`version.rs` 用 `mod version;`
- 如果 `config.rs` 中的函数需要被外部引用，在 `mod.rs` 中 `pub(crate) use config::{...};` 重新导出
- 注册新语言: `main.rs` 中 `registry.register(...)` + `language/mod.rs` 中 `pub mod`

## 常用命令

```bash
cargo build --release                # 生产构建
cargo run -- install node 20         # 测试安装
cargo run -- list-remote node        # 查看远程版本
cargo run -- use node 20.14.0        # 切换版本
cargo run -- env                     # 查看环境变量
cargo clippy --all-targets           # clippy 检查
cargo test --all-targets             # 测试
```
