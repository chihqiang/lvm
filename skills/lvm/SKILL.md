---
name: lvm
description: LVM (Language Version Manager) — Rust CLI tool for managing multiple language versions (Node, Go, Java, Python, Dart, Flutter). Use when working on the LVM codebase, adding new languages, or fixing bugs.
---

# LVM — Language Version Manager

跨平台语言版本管理工具，支持 Node、Go、Java、Python、Dart、Flutter。

## 快速导航

| 文档 | 说明 |
|------|------|
| [项目结构](references/project-structure.md) | 目录树、每个文件的职责 |
| [核心架构](references/architecture.md) | Language trait、注册流程、报告系统、下载安装流程 |
| [如何新增语言](references/add-new-language.md) | 三步模板（config/version/mod），含完整代码 |
| [代码规范](references/coding-rules.md) | 代码风格、Git/PR 规范、CI 检查、常用命令 |
| [工作流与机制](references/workflow.md) | 符号链接、PATH、Shell 自动切换、版本解析链 |
| [环境变量与配置](references/env-and-config.md) | 所有 `LVM_*` 变量、`.lvmrc`/`.nvmrc` 格式、离线模式 |
| [关键设计决策](references/design-decisions.md) | Arch fallback、OnceLock、消息缓冲、Docker 构建等为什么这么做 |

## 现有语言一览

| 语言 | 版本前缀 | arch fallback | 格式 | 镜像源 ENV |
|------|---------|--------------|------|-----------|
| Node | `"v"` | `arm64`→`x64` | tar.gz/zip | `LVM_NODE_MIRROR` |
| Go | `"v"` | `arm64`→`amd64` | tar.gz/zip | `LVM_GO_MIRROR` |
| Java | `""` | `aarch64`→`x64` | tar.gz/zip | `LVM_JAVA_MIRROR` |
| Python | `""` | `aarch64`→`x86_64` | tar.gz | `LVM_PYTHON_MIRROR` |
| Dart | `""` | `arm64`→`x64` | zip (全平台) | `LVM_DART_MIRROR` |
| Flutter | `""` | `arm64`→`x64` | zip (全平台) | `LVM_FLUTTER_MIRROR` |
