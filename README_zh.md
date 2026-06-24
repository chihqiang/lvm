# lvm

Rust 实现的多运行时版本管理工具，跨平台单二进制，统一管理各类运行时版本，支持全局 / 项目自动切换，内置镜像加速，环境隔离无系统污染。

[![Check](https://github.com/chihqiang/lvm/actions/workflows/check.yml/badge.svg)](https://github.com/chihqiang/lvm/actions/workflows/check.yml)
[![lvm version](https://img.shields.io/github/v/release/chihqiang/lvm?color=yellow)](https://github.com/chihqiang/lvm/releases)
[![Rust](https://img.shields.io/badge/rust-stable-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/github/license/chihqiang/lvm)](https://github.com/chihqiang/lvm)
[![Last commit](https://img.shields.io/github/last-commit/chihqiang/lvm)](https://github.com/chihqiang/lvm)

**语言**: [English](README.md)

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | bash
```

脚本自动检测系统架构，下载对应二进制并安装到 `/usr/local/bin/lvm`。

或从 [Releases](https://github.com/chihqiang/lvm/releases) 手动下载对应平台的二进制，放入 PATH 即可。

推荐在 shell 配置文件中添加：

```bash
# ~/.bashrc 或 ~/.zshrc
eval "$(lvm env)"
eval "$(lvm hook)"
```

`lvm env` 设置 PATH 和 GOPATH，`lvm hook` 启用进入目录时自动切换版本（当目录包含 `.lvmrc` 时）。

## Quick Start

### Node.js

```bash
# 查看可安装版本
lvm list-remote node          # 所有版本
lvm list-remote node --lts    # 仅 LTS 版本

# 安装指定版本
lvm install node 22           # 安装最新的 22.x.x
lvm install node 20.0.0       # 安装精确版本
lvm install node --lts        # 安装最新 LTS
lvm install node --lts=iron   # 安装指定 LTS 代号

# 列出已安装版本
lvm list node

# 切换使用版本
lvm use node 22
lvm use node                  # 不指定版本：.lvmrc → 默认别名 → 最新版

# 切换后同时写入 .lvmrc（项目级锁定）
lvm use node 22 --save

# 查看当前版本
lvm current node
lvm current                   # 查看所有语言当前版本
```

### Go

```bash
# 查看可安装版本
lvm list-remote go

# 安装指定版本
lvm install go 1.22
lvm install go 1.21.0

# 列出已安装版本
lvm list go

# 切换使用版本
lvm use go 1.22.0

# 查看当前版本
lvm current go
```

## Commands

| Command | Description |
|---------|-------------|
| `lvm install [language] [version]` | 安装指定版本，支持 `--lts`、`--save`、`--no-default`、`--offline`、`--reinstall-packages-from` |
| `lvm uninstall <language> <version>` | 卸载已安装版本 |
| `lvm use [language] [version]` | 切换当前版本，不指定 version 时自动按 `.lvmrc` → 默认别名 → 最新版查找。支持 `--save`、`--no-default` |
| `lvm list <language>` | 列出已安装版本（标记 current/default） |
| `lvm list-remote <language>` | 列出可安装版本，支持 `--lts` 过滤仅 LTS |
| `lvm current [language]` | 显示当前使用版本，不指定 language 显示所有语言 |
| `lvm alias <language>` | 列出该语言所有别名 |
| `lvm alias <language> <name>` | 查看指定别名对应的版本 |
| `lvm alias <language> <name> <version>` | 设置别名（version 支持 semver、`system`、`lts/*`、semver 范围） |
| `lvm unalias <language> <name>` | 删除别名 |
| `lvm which <language> [version]` | 显示指定版本二进制路径（默认当前） |
| `lvm env` | 输出 shell 环境变量设置脚本（LVM_HOME、GOPATH、PATH） |
| `lvm env --shell <bash\|zsh\|fish>` | 输出对应 shell 的补全脚本 |
| `lvm hook` | 输出 bash/zsh 自动切换 hook（进入含 `.lvmrc` 的目录自动切换） |
| `lvm cache dir` | 显示下载缓存目录 |
| `lvm cache clear` | 清空下载缓存 |
| `lvm debug` | 显示调试信息（LVM_HOME、PATH、已注册语言、当前版本等） |

### Version Resolution

`lvm use` 和 `lvm install` 支持多种版本指定方式：

| 语法 | 示例 | 说明 |
|------|------|------|
| 完整 semver | `20.14.0`、`1.22.0` | 精确版本号 |
| 部分版本 | `22`、`20.18`、`1.22` | 自动匹配最新匹配版本 |
| LTS 最新 | `--lts` | 安装或使用最新 LTS 版本 |
| LTS 代号 | `--lts=iron` | 指定 LTS 代号（如 iron、jod） |
| LTS 语法 | `lts/*`、`lts/iron`、`lts/-1` | 作为 version 参数传入（`-1` = 次新 LTS line） |
| system | `system` | 使用系统已安装的版本（移除 lvm 符号链接） |

## Configuration

### `.lvmrc` — 项目级版本锁定

在项目根目录创建 `.lvmrc`，按 `language=version` 格式写入：

```
node=22.0.0
go=1.22.3
```

支持 `#` 注释和空行。多语言可写在同一文件中。

lvm 进入该目录时（需在 shell 配置中 `eval "$(lvm hook)"`）自动切换所有声明的版本。

使用 `--save` / `-w` 可在安装或切换后自动写入：

```bash
lvm install node 22 --save
lvm use go 1.22 --save
```

### Aliases — 自定义版本名

```bash
lvm alias node default 22        # 设置默认版本
lvm alias go stable 1.22.0       # 设置别名
lvm alias node                   # 列出所有别名
lvm alias node default           # 查看 default 别名
lvm unalias node stable          # 删除别名
```

### `default-packages` — 安装后自动安装的全局包（仅 Node.js）

在本 `~/.lvm/default-packages` 中每行写一个包名（支持 `#` 注释）：

```
# 每次安装 Node.js 后自动安装
pnpm
typescript
eslint
```

## Features

- **多语言**：Node.js、Go 等，插件式架构易于扩展
- **镜像加速**：通过 `LVM_NODE_MIRROR`、`LVM_GO_MIRROR` 环境变量配置镜像源
- **安全校验**：下载后自动验证 SHA256 校验和
- **零系统污染**：版本隔离在 `~/.lvm`，不修改系统目录
- **Per-version 隔离**：每个版本的包完全隔离。`go install` 安装到 `$GOPATH/bin`（指向当前版本），`npm install -g` 安装到版本目录，切换版本后不共享
- **符号链接切换**：无损、原子化的版本切换
- **离线模式**：`--offline` 仅使用缓存
- **Shell 自动切换**：`lvm hook` 输出 bash/zsh hook 脚本，进入含 `.lvmrc` 的目录时自动切换版本

### 镜像源配置

```bash
# Node.js
export LVM_NODE_MIRROR=https://mirrors.ustc.edu.cn/node/

# Go
export LVM_GO_MIRROR=https://mirrors.aliyun.com/golang/
```

## Shell Integration

将以下内容加入你的 shell 配置（`~/.bashrc` 或 `~/.zshrc`）：

```bash
eval "$(lvm env)"    # PATH + GOPATH 设置
eval "$(lvm hook)"   # .lvmrc 自动切换 hook
```

- **`lvm env`**：输出 `LVM_HOME`、`GOPATH`、`PATH` 环境变量。Windows 输出 cmd.exe 语法。
- **`lvm hook`**：输出 bash/zsh 自动切换脚本。bash 通过 `PROMPT_COMMAND`，zsh 通过 `chpwd` 钩子，进入含 `.lvmrc` 的目录时自动执行 `lvm use`。Windows 上不可用，需手动配置 PATH。
- **`lvm env --shell bash|zsh|fish`**：输出命令补全脚本。
- **Node.js**：npm 全局包安装到对应版本目录，切换版本后不共享。
- **Go**：`GOPATH` 自动设为 `$LVM_HOME/current/go/packages`（符号链接动态指向当前版本），`go install` 安装的二进制与系统和其他版本隔离。

## Storage Layout

```bash
~/.lvm/
├── bin/                  # 全局符号链接（加入 PATH）
│   ├── node -> current/node/bin/node
│   └── go   -> current/go/bin/go
├── current/
│   ├── node -> ../node/v22.0.0   # 当前活动 Node 版本
│   └── go   -> ../go/v1.22.0     # 当前活动 Go 版本
├── node/                 # 已安装的 Node.js 版本
│   ├── v20.18.0/         # npm install -g → lib/node_modules/（per-version）
│   └── v22.0.0/
├── go/                   # 已安装的 Go 版本
│   ├── v1.21.0/
│   │   └── packages/bin/ # go install 安装的二进制（per-version）
│   └── v1.22.0/
│       └── packages/bin/
├── aliases/              # 别名配置
│   ├── node/
│   │   └── default -> 22
│   └── go/
│       └── stable -> 1.22.0
├── cache/                # 版本列表缓存
├── downloads/            # 下载缓存（安装包归档文件）
└── default-packages      # Node.js 自动安装列表
```

## Comparison

| Feature | nvm | fnm | gvm | lvm |
|---------|-----|-----|-----|-----|
| 语言 | Shell | Rust | Shell | Rust |
| 跨平台 | 仅 Unix | ✓ | 仅 Unix | ✓ |
| 性能 | 慢 | 快 | 慢 | 快 |
| 多语言 | 仅 Node | 仅 Node | 仅 Go | ✓ 插件化扩展 |
| 镜像加速 | ✗ | ✗ | ✗ | ✓ 内置 |
| SHA256 校验 | ✗ | ✗ | ✗ | ✓ |
| 离线模式 | ✗ | ✗ | ✗ | ✓ |
| 自动切换 | ✗ | ✗ | ✗ | ✓ 内置 hook |
| 项目级锁定 | .nvmrc | .node-version | ✗ | .lvmrc |