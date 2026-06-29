# 核心架构

## Language Trait

`src/language/language_trait.rs` 定义了所有语言必须实现的接口：

```rust
pub trait Language {
    fn name(&self) -> &str;                      // 语言名称（注册 key）
    fn install(&self, version: Option<&str>) -> Result<String>;
    fn latest_version(&self) -> Result<String>;

    // 以下有默认实现，可按需覆盖：
    fn version_prefix(&self) -> &'static str { "v" }
    fn binary_name(&self) -> &str { self.name() }
    fn subdir_name(&self) -> &str { self.name() }
    fn list_remote_versions(&self) -> Result<Vec<String>>;
    fn is_installed(&self, version_dir: &Path) -> bool;
    fn env_extra_paths(&self) -> Vec<PathBuf>;
    fn env_extra_vars(&self) -> Vec<(&'static str, PathBuf)>;
    fn package_manager_binary(&self) -> Option<&'static str>;
    fn post_install(&self, _version: &str) -> Result<()>;
    fn post_switch(&self, _version: &str) -> Result<()>;
}
```

默认实现的方法（无需覆盖）：
- `version_dir()` — 拼接 `{lvm_dir}/{prefix}{version}`
- `current_link()` / `bin_link()` — symlink 路径
- `is_installed()` — 检查 `version_dir/bin/{binary_name}` 是否存在
- `use_version()` — 创建 symlink + 写 default 别名
- `list_installed()` / `format_installed()` — 已安装版本列表
- `current_version()` / `uninstall()` / `binary_path()`

## 注册流程

`src/main.rs` 注册：

```rust
registry.register(Box::new(language::dart::DartLanguage));
```

`src/language/mod.rs` 声明：

```rust
pub mod dart;
```

`LanguageRegistry` 使用 `HashMap<String, Box<dyn Language>>` 按 name 索引。

## 消息报告系统

所有输出使用 `report("message")` / `report(format!(...))` 缓冲到全局 `Vec<String>`，然后在关键节点调用 `flush_reports_to_stdout()` 一次性输出。预定义的报告函数在 `core/report.rs`：

- `report_already_installed(name, version)`
- `report_non_native_arch(os, arch)`
- `report_fallback(from, to)`
- `report_verifying_checksum()`
- `report_checksum_verified()`

## 下载与安装流程

`download_and_install(url, tar_path, version, version_dir, display_name, verify_fn)`：

1. `ensure_downloaded()` — 检查缓存 → 下载 → 校验
2. 解压到临时目录 `.v{version}.tmp-{pid}/`
3. 自动 `strip_top_level` 去掉 archive 顶层目录
4. `rename()` 原子替换到 `version_dir`
5. 失败时清理临时目录

`verify_fn` 闭包 `|path: &Path| -> Result<()>` 用于校验下载文件：
- zip 格式用 `ZipArchive::new(file)` 检查完整性
- tar.gz 可传 `|_| Ok(())`

### 解压自动检测

`extract.rs` 根据文件扩展名自动分流：

```rust
if archive_path.extension() == "zip" {
    extract_zip(...)    // zip::ZipArchive
} else {
    extract_tarball(...) // tar::Archive + flate2::GzDecoder
}
```

### 版本列表缓存

`fetch_with_cache(&cache_file, fetch_fn)` 自动缓存 5 分钟（TTL 在 `config::cache_ttl()`）。
