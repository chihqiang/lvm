# 关键设计决策

## Arch Fallback 策略

**问题**: ARM64（Apple Silicon、ARM Linux）用户安装语言版本时，部分旧版本没有 ARM64 构建。

**决策**: 先试原生 arch，下载失败后自动回退到 x64/amd64。

```rust
let archs: &[&str] = if config::target_arch() != "x64" {
    &[config::target_arch(), "x64"]
} else {
    &[config::target_arch()]
};
```

**理由**:

- 跨平台统一，不引入平台特定代码
- 回退消息使用 `"non-native arch"` 而非平台特定名称（如 `"Rosetta"`）
- macOS 上 x64 二进制可通过 Rosetta 2 运行，Linux 上可通过 qemu 或其他模拟器运行

## 为什么不用 `serde` derive

**问题**: `Cargo.toml` 只依赖了 `serde_json`，没有 `serde` 包。

**决策**: 手动使用 `serde_json::Value` 解析 JSON，而非 `#[derive(Deserialize)]`。

**理由**:

- 减少编译依赖和编译时间
- API 返回结构不稳定时不会被反序列化错误阻塞
- 很多场景只需要取一两个字段，`Value` 更灵活

## 为什么用 `OnceLock` 而非 `lazy_static` 或 `LazyLock`

**决策**: 使用 `std::sync::OnceLock`（Rust 1.70+ stable）做单例初始化。

**理由**:

- 标准库内置，零依赖
- 在 Rust edition 2024 中 `LazyLock` 已稳定，但 `OnceLock` 更简洁直观
- 不需要 `once_cell` 或 `lazy_static` 第三方包

## 消息缓冲系统

**问题**: 安装过程中多个步骤需要输出消息（下载中、解压中、校验中），但输出混杂在进度条和错误信息中会混乱。

**决策**: 全局 `Mutex<Vec<String>>` 缓冲，关键节点 `flush()` 统一输出。

```text
report("Downloading...") → push to Vec
report("Extracting...")  → push to Vec
flush_reports_to_stdout() → drain + writeln
```

**理由**:

- 避免进度条和文本输出冲突
- 错误发生时可以清理缓冲，不输出不完整信息
- 单元测试中可以检查 report 内容

## 版本列表缓存

**问题**: 每次列出远程版本都请求 API，速度慢且可能触发限流。

**决策**: `fetch_with_cache()` 自动缓存 5 分钟到 `~/.lvm/cache/`。

```rust
if let Ok(modified) = meta.modified()
    && let Ok(elapsed) = modified.elapsed()
    && elapsed < cache_ttl()  // 5 分钟
{
    return fs::read_to_string(cache_file);
}
```

**理由**:

- 减少网络请求
- 本地快速响应
- TTL 适中，不会用过时版本信息

## 为什么用 `OnceLock<String>` 而非 `OnceLock<&str>`

**问题**: 环境变量默认值是静态字符串，但在运行时可能被覆盖。

**决策**: 使用 `OnceLock<String>` 存储，`get_or_init()` 初始化时读取环境变量，允许默认值。

**理由**:

- 环境变量是动态的，需要 `String` 而非 `&'static str`
- `OnceLock` 保证线程安全的一次性初始化
- 运行时不可变，避免并发问题

## 为什么不用 `thiserror`

**决策**: 使用 `anyhow` 而非 `thiserror` 做错误处理。

**理由**:

- 项目是 CLI 工具而非库，不需要自定义错误类型
- `anyhow::Context` 和 `bail!` 在 CLI 场景中更便捷
- `.with_context(|| format!(...))` 提供了丰富的错误上下文
- `thiserror` 适合库项目的公共错误类型定义

## 符号链接原子替换

**问题**: 直接覆盖 symlink 可能产生中间状态。

**决策**: 先创建临时 symlink，再 `rename()` 原子覆盖。

```rust
let tmp = temp_symlink_path(dst)?;
create_symlink(src, &tmp)?;
fs::rename(&tmp, dst)?;
```

**理由**:

- `rename()` 是原子操作（POSIX 保证同一文件系统内原子性）
- 临时文件命名 `.v{version}.tmp-{pid}` 避免冲突
- 失败时临时文件可清理，不影响原 symlink

## Docker 构建策略

**问题**: 发布二进制需要匹配目标系统的 glibc 版本。

**决策**: Dockerfile.build 使用 `rust:bookworm`（glibc 2.36）作为 builder，保持与 runtime 一致的 glibc。

**理由**:

- 避免 `GLIBC_2.xx` not found 错误
- 多阶段构建（builder → runner）减小镜像体积
- install.sh 直接下载预编译二进制，不依赖 Docker
