/// 统一输出模块，所有命令层日志经此模块
use crate::plugin;

pub(crate) fn info(msg: impl AsRef<str>) {
    println!("{}", msg.as_ref());
}

pub(crate) fn warn(msg: impl AsRef<str>) {
    eprintln!("Warning: {}", msg.as_ref());
}

/// 排出 plugin 层缓存的日志消息
pub(crate) fn flush_plugin() {
    for msg in plugin::drain_reports() {
        println!("{msg}");
    }
}
