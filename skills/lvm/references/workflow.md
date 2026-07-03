# 工作流与机制

## 符号链接架构

版本切换通过两层符号链接实现（`src/core/fslink.rs`）：

```text
~/.lvm/{lang}/v{version}/bin/{lang}     # 实际版本安装目录
     ↑  current_link 指向
~/.lvm/current/{lang}/bin/{lang}         # 当前版本（symlink）
     ↑  bin_link 指向
~/.lvm/bin/{lang}                        # 用户 PATH 入口（symlink）
```

路径布局：

- `~/.lvm/{name}/v{version}/bin/{name}` — 版本安装目录
- `~/.lvm/current/{name}` → `~/.lvm/{name}/v{version}`（当前版本）
- `~/.lvm/bin/{name}` → `~/.lvm/current/{name}/bin/{name}`（PATH 入口）

切换流程（`use_version_symlinks()`）：

1. 创建临时 symlink `.v20.14.0.tmp-{pid}`
2. `rename()` 原子替换目标 symlink
3. 旧 symlink 自动被覆盖

## Shims 与 PATH

`lvm env` 输出包含（`src/commands/env/mod.rs`）：

```bash
export LVM_HOME="$HOME/.lvm"
export KOTLIN_HOME="$HOME/.lvm/current/kotlin"   # 各语言的 env_extra_vars
export JAVA_HOME="$HOME/.lvm/current/java"
export GOPATH="$HOME/.lvm/current/go/packages"    # Go 额外路径
export FLUTTER_HOME="$HOME/.lvm/current/flutter"
export DART_HOME="$HOME/.lvm/current/dart"
export PUB_CACHE="$HOME/.lvm/current/dart/pub-cache"
export PATH="$HOME/.lvm/current/node/bin:$HOME/.lvm/current/go/packages/bin:$HOME/.lvm/bin:$PATH"
```

PATH 条目拼接顺序（仅对已激活版本的语言生效）：

1. 每个语言的 `current/{name}/bin`（自动）
2. 每个语言的 `env_extra_paths()`（如 Go 的 `current/go/packages/bin`）
3. `$LVM_HOME/bin`（最后兜底）

## Shell 自动切换（Hook 系统）

`lvm hook {zsh,bash,fish}` 生成 shell 函数（`src/commands/hook/mod.rs`）：

- **Bash/PROMPT_COMMAND**：每次提示符显示前检测 `.lvmrc` 变化
- **Zsh/chpwd**：目录切换时触发 `lvm use`
- **Fish/$PWD**：`--on-variable PWD` 事件

不区分 `.lvmrc` 还是 `.nvmrc`，统一走 `lvm use` 命令自动解析。
Hook 脚本使用 `std::env::current_exe()` 获取 lvm 二进制真实路径，无论安装在哪里都能正确工作。

## 命令工作流

### `lvm install node 20`

```text
dispatch.rs → resolve_install_args() 解析参数
  → commands::install() → language.install("20")
    → resolve_version() 解析 20 → "20.14.0"
    → download_and_install() 下载+解压到 ~/.lvm/node/v20.14.0/
  → language.use_version() 创建 symlink
  → language.post_install() 安装后钩子（如 npm 默认包）
  → 如果 —save，写入 .lvmrc
```

### `lvm use`（不带参数）

```text
dispatch.rs → 没有 language/version 参数
  → find_lvmrc() 向上遍历查找 .lvmrc
  → 如果找到，解析全部 language=version 对
  → 如果没找到，尝试 read_nvmrc()
  → 对每对执行 use_version()
```

## 版本解析链（`use_version/mod.rs`）

当没有指定版本时（`lvm use node`），优先级：

1. `.lvmrc`（所有语言通用，`lvmrc::read_lvmrc_version()`）
2. `rc_version()` — 各语言 RC 文件（Node 的 `nvmrc::resolve_nvmrc_version()`）
3. `default` 别名（`lvm alias node default 20`）
4. 远程最新版本

## 调试与诊断

`lvm debug` 输出（`src/commands/debug/mod.rs`）：

```text
lvm v0.0.6
OS:          macos
Arch:        aarch64
LVM_HOME:    /Users/user/.lvm
Downloads:   /Users/user/.lvm/downloads
Cache:       /Users/user/.lvm/cache

Registered languages:
  node:    current=20.14.0
  go:      current=1.22.3
  java:    current=21.0.3
  python:  current=3.12.4
  dart:    current=3.6.0
  flutter: current=3.29.0
  kotlin:  current=2.0.0
  rust:    current=1.82.0

PATH entries:
  /Users/user/.lvm/current/node/bin ← lvm
  /Users/user/.lvm/current/go/packages/bin ← lvm
  /Users/user/.lvm/bin ← lvm
  /opt/homebrew/bin [has node, go]
```
