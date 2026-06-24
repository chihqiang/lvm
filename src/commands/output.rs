pub(crate) fn info(msg: impl AsRef<str>) {
    println!("{}", msg.as_ref());
}

pub(crate) fn warn(msg: impl AsRef<str>) {
    eprintln!("Warning: {}", msg.as_ref());
}
