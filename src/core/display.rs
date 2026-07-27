//! Display and formatting constants

/// Check if stdout supports color output (cached)
pub fn use_color() -> bool {
    use std::sync::OnceLock;
    static IS_TTY: OnceLock<bool> = OnceLock::new();
    *IS_TTY.get_or_init(|| {
        use std::io::IsTerminal;
        std::io::stdout().is_terminal()
    })
}

/// Bold green ANSI code (highlighting the current version)
pub const COLOR_GREEN_BOLD: &str = "\x1b[1;32m";

/// Yellow ANSI code (for alternative versions)
pub const COLOR_YELLOW: &str = "\x1b[33m";

/// Green ANSI code (for installed versions)
pub const COLOR_GREEN: &str = "\x1b[32m";

/// Cyan ANSI code (for LTS versions)
pub const COLOR_CYAN: &str = "\x1b[36m";

/// Bold ANSI code
const COLOR_BOLD: &str = "\x1b[1m";

/// Reset ANSI code
pub const COLOR_RESET: &str = "\x1b[0m";

/// LTS version marker (in version listings)
pub const LTS_MARKER: &str = "(LTS:";

/// Check mark symbol (indicates installed)
pub const INSTALLED_CHECK_MARK: &str = "\u{2713}";

/// Colored check mark (bold)
pub fn colored_check_mark() -> &'static str {
    use std::sync::OnceLock;
    static COLORED: OnceLock<String> = OnceLock::new();
    COLORED.get_or_init(|| format!("{COLOR_BOLD}{INSTALLED_CHECK_MARK}{COLOR_RESET}"))
}
