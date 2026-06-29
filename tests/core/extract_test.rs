use std::fs;
use std::io::Write;
use std::path::Path;

/// 创建简单的 tar.gz 归档（用于测试提取）
fn create_test_tarball(path: &Path, filename: &str, content: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::none());
    let mut tar = tar::Builder::new(encoder);

    // 添加一个顶级目录前缀
    let arc_path = format!("top-level/{filename}");
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, &arc_path, content).unwrap();

    let encoder = tar.into_inner().unwrap();
    encoder.finish().unwrap();
}

/// 创建简单的 zip 归档
fn create_test_zip(path: &Path, filename: &str, content: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    let arc_path = format!("top-level/{filename}");
    zip.start_file(&arc_path, options).unwrap();
    zip.write_all(content).unwrap();
    zip.finish().unwrap();
}

#[test]
fn test_extract_tarball() {
    let dir = tempfile::tempdir().unwrap();
    let tar_path = dir.path().join("test.tar.gz");
    let out_dir = dir.path().join("extracted");

    create_test_tarball(&tar_path, "hello.txt", b"hello world");
    lvm::core::extract::extract_archive(&tar_path, &out_dir).unwrap();

    let extracted = out_dir.join("hello.txt");
    assert!(extracted.exists());
    assert_eq!(fs::read_to_string(extracted).unwrap(), "hello world");
}

#[test]
fn test_extract_zip() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("test.zip");
    let out_dir = dir.path().join("extracted");

    create_test_zip(&zip_path, "hello.txt", b"hello from zip");
    lvm::core::extract::extract_archive(&zip_path, &out_dir).unwrap();

    let extracted = out_dir.join("hello.txt");
    assert!(extracted.exists());
    assert_eq!(fs::read_to_string(extracted).unwrap(), "hello from zip");
}

#[test]
fn test_extract_invalid_archive() {
    let dir = tempfile::tempdir().unwrap();
    let bad_path = dir.path().join("bad.tar.gz");
    let out_dir = dir.path().join("extracted");

    fs::write(&bad_path, b"not an archive").unwrap();
    let result = lvm::core::extract::extract_archive(&bad_path, &out_dir);
    assert!(result.is_err());
}
