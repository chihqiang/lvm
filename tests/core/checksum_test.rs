use std::fs;
use std::io::Write;

fn create_temp_file(content: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("testfile");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(content).unwrap();
    (dir, path)
}

#[test]
fn test_sha256_of_known_value() {
    // SHA256 of empty string
    let (_dir, path) = create_temp_file(b"");
    let hash = lvm::core::checksum::sha256_of(&path).unwrap();
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_sha256_of_hello() {
    // SHA256 of "hello"
    let (_dir, path) = create_temp_file(b"hello");
    let hash = lvm::core::checksum::sha256_of(&path).unwrap();
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn test_verify_sha256_valid() {
    let (_dir, path) = create_temp_file(b"hello");
    lvm::core::checksum::verify_sha256(
        &path,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
    )
    .unwrap();
}

#[test]
fn test_verify_sha256_mismatch() {
    let (_dir, path) = create_temp_file(b"hello");
    let result = lvm::core::checksum::verify_sha256(
        &path,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(result.is_err());
}

#[test]
fn test_verify_sha256_invalid_hex() {
    let (_dir, path) = create_temp_file(b"hello");
    // too short
    let result = lvm::core::checksum::verify_sha256(&path, "abc123");
    assert!(result.is_err());
    // non-hex chars
    let result = lvm::core::checksum::verify_sha256(
        &path,
        "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
    );
    assert!(result.is_err());
}
