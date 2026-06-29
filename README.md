# lvm

A multi-runtime version manager written in Rust. Cross-platform single binary, unified management of various runtime versions, supports global/project automatic switching, built-in mirror acceleration, and environment isolation with no system pollution.

[![Check](https://github.com/chihqiang/lvm/actions/workflows/check.yml/badge.svg)](https://github.com/chihqiang/lvm/actions/workflows/check.yml)
[![HitCount](https://views.whatilearened.today/views/github/chihqiang/lvm.svg)](https://github.com/chihqiang/lvm)
[![lvm version](https://img.shields.io/github/v/release/chihqiang/lvm?color=yellow)](https://github.com/chihqiang/lvm/releases)
[![Rust](https://img.shields.io/badge/rust-stable-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/github/license/chihqiang/lvm)](https://github.com/chihqiang/lvm)
[![Last commit](https://img.shields.io/github/last-commit/chihqiang/lvm)](https://github.com/chihqiang/lvm)


**Language**: [中文](README_zh.md)

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | bash
```

The script automatically detects the system architecture, downloads the corresponding binary, and installs it to `/usr/local/bin/lvm`.

Alternatively, manually download the binary for your platform from [Releases](https://github.com/chihqiang/lvm/releases) and place it in your PATH.

It is recommended to add the following to your shell configuration:

```bash
# ~/.bashrc or ~/.zshrc
eval "$(lvm env)"
eval "$(lvm hook)"
```

`lvm env` sets up PATH and GOPATH, and `lvm hook` enables automatic version switching when entering a directory (when the directory contains `.lvmrc` or `.nvmrc` files).

## Quick Start

### Node.js

```bash
# List installable versions
lvm list-remote node          # All versions
lvm list-remote node --lts    # LTS versions only

# Install a specific version
lvm install node 22           # Install the latest 22.x.x
lvm install node 20.0.0       # Install an exact version
lvm install node --lts        # Install the latest LTS
lvm install node --lts=iron   # Install a specific LTS codename

# List installed versions
lvm list node

# Switch version
lvm use node 22
lvm use node                  # Without version: .nvmrc → .lvmrc → default alias → latest

# Switch and write .lvmrc (project-level lock)
lvm use node 22 --save

# Show current version
lvm current node
lvm current                   # Show current versions for all languages
```

### Go

```bash
# List installable versions
lvm list-remote go

# Install a specific version
lvm install go 1.22
lvm install go 1.21.0

# List installed versions
lvm list go

# Switch version
lvm use go 1.22.0

# Show current version
lvm current go
```

## Commands

| Command | Description |
|---------|-------------|
| `lvm install [language] [version]` | Install a version. Without arguments, installs all from `.lvmrc`; falls back to `.nvmrc` if no `.lvmrc`. Supports `--lts`, `--save`, `--no-default`, `--offline`, `--reinstall-packages-from` |
| `lvm uninstall <language> <version>` | Uninstall a version |
| `lvm use [language] [version]` | Switch to a version. Without version for Node, resolves via `.nvmrc` → `.lvmrc` → default alias → latest. Supports `--save`, `--no-default` |
| `lvm list <language>` | List installed versions (marks current/default) |
| `lvm list-remote <language>` | List installable versions. Supports `--lts` to filter LTS only |
| `lvm current [language]` | Show the active version. Without language, shows all languages |
| `lvm alias <language>` | List all aliases for a language |
| `lvm alias <language> <name>` | Show the version for a specific alias |
| `lvm alias <language> <name> <version>` | Set an alias (version supports semver, `system`, `lts/*`, semver ranges) |
| `lvm unalias <language> <name>` | Delete an alias |
| `lvm which <language> [version]` | Show the binary path for a version (defaults to current) |
| `lvm env` | Output shell environment variable setup script (LVM_HOME, GOPATH, PATH) |
| `lvm env --shell <bash\|zsh\|fish>` | Output shell completion script |
| `lvm hook [--shell bash|zsh|fish|powershell]` | Output shell auto-switch hook (bash: `PROMPT_COMMAND`, zsh: `chpwd`, fish: `--on-variable PWD`, powershell: `prompt`) |
| `lvm prune <language> [--keep N]` | Remove all but N newest versions (skips current/default). Default keep=3 |
| `lvm cache dir` | Show the download cache directory |
| `lvm cache clear` | Clear the download cache |
| `lvm debug` | Show debug info (LVM_HOME, PATH, registered languages, current versions, etc.) |

### Version Resolution

`lvm use` and `lvm install` support multiple ways to specify a version:

| Syntax | Example | Description |
|--------|---------|-------------|
| Full semver | `20.14.0`, `1.22.0` | Exact version |
| Partial version | `22`, `20.18`, `1.22` | Auto-resolves to the latest matching version |
| Latest LTS | `--lts` | Install or use the latest LTS version |
| LTS codename | `--lts=iron` | Specify an LTS codename (e.g., iron, jod) |
| LTS syntax | `lts/*`, `lts/iron`, `lts/-1` | Pass as version argument (`-1` = second-latest LTS line) |
| system | `system` | Use the system-installed version (remove lvm symlink) |

## Configuration

### Project-level version lock

Create a `.lvmrc` file in your project root with the format `language=version`:

```
node=22.0.0
go=1.22.3
```

Supports `#` comments and blank lines. Multiple languages can be specified in the same file.

**`.nvmrc` compatibility**: lvm also reads `.nvmrc` files. When using `lvm use node` (without specifying a version), `.nvmrc` is checked before `.lvmrc`. This ensures seamless migration from nvm/fnm — just keep your existing `.nvmrc` files.

Run `lvm install` without arguments to install all versions declared in `.lvmrc` at once.

lvm automatically switches all declared versions when entering the directory (requires `eval "$(lvm hook)"` in your shell config). `.nvmrc`-only projects are also supported for automatic switching.

Use `--save` / `-w` to automatically write `.lvmrc` after install or switch:

```bash
lvm install node 22 --save
lvm use go 1.22 --save
```

### Aliases — Custom version names

```bash
lvm alias node default 22        # Set the default version
lvm alias go stable 1.22.0       # Set an alias
lvm alias node                   # List all aliases
lvm alias node default           # View the default alias
lvm unalias node stable          # Delete an alias
```

### `--reinstall-packages-from` — Migrate global packages (Node.js only)

When upgrading Node.js versions, reinstall all global packages from a previous version:

```bash
lvm install node 22 --reinstall-packages-from=20.14.0
```

This lists global packages from the old version and installs them on the new one (skips `npm`, `corepack`).

### `default-packages` — Auto-installed global packages (Node.js only)

Write one package per line in `~/.lvm/default-packages` (supports `#` comments). Use `package@version` to pin a specific version for compatibility:

```
# Auto-installed after each Node.js installation
pnpm@8.15.9
typescript
eslint
```

## Features

- **Multi-language**: Node.js, Go, and more with a plugin-based architecture for easy extension
- **Mirror acceleration**: Configure mirror sources via `LVM_NODE_MIRROR` and `LVM_GO_MIRROR` environment variables
- **Security verification**: Automatically verifies SHA256 checksums after download
- **Zero system pollution**: Versions are isolated in `~/.lvm`, no system directories are modified
- **Per-version isolation**: Packages for each version are fully isolated. `go install` goes to `$GOPATH/bin` (points to current version), `npm install -g` installs to the version directory — no sharing between versions
- **Symlink switching**: Lossless, atomic version switching
- **Offline mode**: `--offline` uses cache only
- **Shell auto-switch**: `lvm hook` outputs hook scripts for bash/zsh/fish/powershell that auto-switch versions when entering directories with `.lvmrc` or `.nvmrc`

### Mirror Configuration

```bash
# Node.js
export LVM_NODE_MIRROR=https://mirrors.ustc.edu.cn/node/

# Go
export LVM_GO_MIRROR=https://mirrors.aliyun.com/golang/
```

## Shell Integration

Add the following to your shell configuration (`~/.bashrc` or `~/.zshrc`):

```bash
eval "$(lvm env)"    # PATH + GOPATH setup
eval "$(lvm hook)"   # .lvmrc / .nvmrc auto-switch hook
```

- **`lvm env`**: Outputs `LVM_HOME`, `GOPATH`, `PATH` environment variables. Windows outputs cmd.exe syntax.
- **`lvm hook`**: Outputs auto-switch script. Defaults to detecting the current shell; use `--shell` to override. bash uses `PROMPT_COMMAND`, zsh uses the `chpwd` hook, fish uses `--on-variable PWD`, powershell overrides the `prompt` function — automatically runs `lvm use` when entering a directory containing `.lvmrc` or `.nvmrc`. Not available on Windows (unless `--shell powershell` is explicitly given).
- **`lvm env --shell bash|zsh|fish`**: Outputs command completion scripts.
- **Node.js**: npm global packages are installed to the corresponding version directory; not shared after switching versions.
- **Go**: `GOPATH` is automatically set to `$LVM_HOME/current/go/packages` (symlink dynamically points to the current version). Binaries installed via `go install` are isolated from the system and other versions.

## Storage Layout

```bash
~/.lvm/
├── bin/                  # Global symlinks (added to PATH)
│   ├── node -> current/node/bin/node
│   └── go   -> current/go/bin/go
├── current/
│   ├── node -> ../node/v22.0.0   # Current active Node version
│   └── go   -> ../go/v1.22.0     # Current active Go version
├── node/                 # Installed Node.js versions
│   ├── v20.18.0/         # npm install -g → lib/node_modules/ (per-version)
│   └── v22.0.0/
├── go/                   # Installed Go versions
│   ├── v1.21.0/
│   │   └── packages/bin/ # Binaries installed via go install (per-version)
│   └── v1.22.0/
│       └── packages/bin/
├── aliases/              # Alias configuration
│   ├── node/
│   │   └── default -> 22
│   └── go/
│       └── stable -> 1.22.0
├── cache/                # Version list cache
├── downloads/            # Download cache (installation archives)
└── default-packages      # Node.js auto-install list
```

## Comparison

| Feature | nvm | fnm | gvm | lvm |
|---------|-----|-----|-----|-----|
| Language | Shell | Rust | Shell | Rust |
| Cross-platform | Unix only | ✓ | Unix only | ✓ |
| Performance | Slow | Fast | Slow | Fast |
| Multi-language | Node only | Node only | Go only | ✓ Plugin-based |
| Mirror acceleration | ✗ | ✗ | ✗ | ✓ Built-in |
| SHA256 checksum | ✗ | ✗ | ✗ | ✓ |
| Offline mode | ✗ | ✗ | ✗ | ✓ |
| Auto-switch | ✗ | ✗ | ✗ | ✓ Built-in hook |
| Project-level lock | .nvmrc | .node-version | ✗ | .lvmrc + .nvmrc |
