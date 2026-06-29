# 环境变量与配置

## 环境变量一览

| 变量 | 默认值 | 用途 | 定义位置 |
| ------ | ------ | ------ | ------- |
| `LVM_NODE_MIRROR` | `https://nodejs.org/dist` | Node 下载镜像源 | `node/config.rs` |
| `LVM_GO_MIRROR` | `https://go.dev/dl` | Go 下载镜像源 | `go/config.rs` |
| `LVM_JAVA_MIRROR` | `https://api.adoptium.net/v3` | Java (Adoptium) API | `java/config.rs` |
| `LVM_PYTHON_MIRROR` | `https://github.com/astral-sh/python-build-standalone/releases/download` | Python 下载镜像源 | `python/config.rs` |
| `LVM_PYTHON_TAG` | `20260623` | python-build-standalone 发布 tag | `python/config.rs` |
| `LVM_DART_MIRROR` | `https://storage.googleapis.com/dart-archive` | Dart SDK 下载镜像源 | `dart/config.rs` |
| `LVM_FLUTTER_MIRROR` | `https://storage.googleapis.com/flutter_infra_release/releases` | Flutter SDK 下载镜像源 | `flutter/config.rs` |
| `LVM_INSTALL_DIR` | `/usr/local/bin` | install.sh 安装路径 | `install.sh` |
| `LVM_DOWNLOAD_URL` | `https://github.com/chihqiang/lvm/releases/latest/download` | install.sh 下载 base URL | `install.sh` |

运行时环境变量（不依赖 `lvm env`）：

| 变量 | 说明 |
| ------ | ------ |
| `LVM_HOME` | 由 `lvm env` 设置，指向 `~/.lvm` |
| `{LANG}_HOME` | 由 `lvm env` 设置，如 `DART_HOME`、`FLUTTER_HOME` |
| `PATH` | 由 `lvm env` 追加各语言 `bin/` 目录 |

## `.lvmrc` 文件格式

`.lvmrc` 文件使用 `key=value` 格式，每行一个语言版本映射：

```text
node=20.14.0
go=1.22.3
python=3.12.0
dart=3.12.2
flutter=3.29.0
# 这是注释
# 空行会被忽略
```

- 查找规则：从当前目录向上遍历最多 100 层
- 优先级低于 `.nvmrc`（仅 Node），高于 default 别名
- `lvm install` 和 `lvm use` 支持 `--save` / `-w` 写入

## `.nvmrc` 兼容

`.nvmrc` 文件由 `node/nvmrc.rs` 读取，仅影响 Node。查找规则同 `.lvmrc`（向上遍历），优先级高于 `.lvmrc`。

## 离线模式

`lvm install --offline` 设置离线标志：

- 跳过网络下载，仅使用缓存文件
- 如果缓存不存在则报错
- 断点续传也被禁用

## 镜像源配置模式

所有语言镜像源使用同一模式（`src/core/config.rs` 中的 `OnceLock`）：

```rust
static MIRROR: OnceLock<String> = OnceLock::new();

pub(crate) fn mirror() -> &'static str {
    MIRROR.get_or_init(|| {
        env::var("LVM_{NAME}_MIRROR")
            .unwrap_or_else(|_| "https://default-mirror.com".to_string())
    })
}
```

- 惰性初始化，仅首次调用时读取环境变量
- 运行时不可更改（进程级缓存）
