//! 显示和格式化常量配置
//! 集中管理输出、颜色代码、标记等显示相关的硬编码值

// ─── ANSI 颜色代码 ───

/// 绿色加粗 ANSI 代码（用于强调当前版本）
pub(crate) fn color_green_bold() -> &'static str {
    "\x1b[1;32m"
}

/// 黄色 ANSI 代码（用于备选版本）
pub(crate) fn color_yellow() -> &'static str {
    "\x1b[33m"
}

/// 绿色 ANSI 代码（用于已安装版本）
pub(crate) fn color_green() -> &'static str {
    "\x1b[32m"
}

/// 青色 ANSI 代码（用于 LTS 版本）
pub(crate) fn color_cyan() -> &'static str {
    "\x1b[36m"
}

/// 粗体 ANSI 代码
pub(crate) fn color_bold() -> &'static str {
    "\x1b[1m"
}

/// 重置 ANSI 代码
pub(crate) fn color_reset() -> &'static str {
    "\x1b[0m"
}

// ─── 列表显示标记 ───

/// LTS 版本标记（在版本列表中）
pub(crate) fn lts_marker() -> &'static str {
    "(LTS:"
}

/// 勾号符号 ✓（表示已安装）
pub(crate) fn installed_check_mark() -> &'static str {
    "\u{2713}"
}

/// 带颜色的勾号（粗体）
pub(crate) fn colored_check_mark() -> String {
    format!(
        "{}{}{}",
        color_bold(),
        installed_check_mark(),
        color_reset()
    )
}

// ─── 命令和参数 ───

/// 系统版本关键字
pub(crate) fn system_keyword() -> &'static str {
    "system"
}
