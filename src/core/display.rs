//! 显示和格式化常量配置

/// 检测 stdout 是否支持颜色输出
pub fn use_color() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// 绿色加粗 ANSI 代码（用于强调当前版本）
pub const COLOR_GREEN_BOLD: &str = "\x1b[1;32m";

/// 黄色 ANSI 代码（用于备选版本）
pub const COLOR_YELLOW: &str = "\x1b[33m";

/// 绿色 ANSI 代码（用于已安装版本）
pub const COLOR_GREEN: &str = "\x1b[32m";

/// 青色 ANSI 代码（用于 LTS 版本）
pub const COLOR_CYAN: &str = "\x1b[36m";

/// 粗体 ANSI 代码
const COLOR_BOLD: &str = "\x1b[1m";

/// 重置 ANSI 代码
pub const COLOR_RESET: &str = "\x1b[0m";

/// LTS 版本标记（在版本列表中）
pub const LTS_MARKER: &str = "(LTS:";

/// 勾号符号 ✓（表示已安装）
pub const INSTALLED_CHECK_MARK: &str = "\u{2713}";

/// 带颜色的勾号（粗体）
pub fn colored_check_mark() -> &'static str {
    use std::sync::OnceLock;
    static COLORED: OnceLock<String> = OnceLock::new();
    COLORED.get_or_init(|| format!("{COLOR_BOLD}{INSTALLED_CHECK_MARK}{COLOR_RESET}"))
}
