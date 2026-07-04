use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Returns a fixture from tests/fixtures/generated/darwin/, (re)generating
/// them once per test run. These files carry macOS xattrs and file flags
/// that git can't store, so they aren't checked in — they're left in place
/// after the run for poking at dtls manually.
#[cfg(target_os = "macos")]
fn darwin_fixture(name: &str) -> PathBuf {
    static GENERATE: std::sync::Once = std::sync::Once::new();
    GENERATE.call_once(generate_darwin_fixtures);
    fixture("generated/darwin").join(name)
}

/// Sets `attr` on `path` to the binary-plist encoding of `value`, the format
/// macOS uses for its com.apple.metadata:* attributes.
#[cfg(target_os = "macos")]
fn set_plist_xattr(path: &Path, attr: &str, value: plist::Value) {
    let mut bytes = Vec::new();
    value.to_writer_binary(&mut bytes).unwrap();
    xattr::set(path, attr, &bytes).unwrap();
}

#[cfg(target_os = "macos")]
fn generate_darwin_fixtures() {
    use plist::Value;

    let dir = fixture("generated/darwin");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();

    // Where from (kMDItemWhereFroms): download source URLs.
    let path = dir.join("downloaded-file.txt");
    std::fs::write(&path, "downloaded content\n").unwrap();
    set_plist_xattr(
        &path,
        "com.apple.metadata:kMDItemWhereFroms",
        Value::Array(vec![
            Value::String("https://example.com/downloaded.zip".into()),
            Value::String("https://example.com/page.html".into()),
        ]),
    );

    // Finder comment (kMDItemFinderComment).
    let path = dir.join("commented-file.txt");
    std::fs::write(&path, "a file with a Finder comment\n").unwrap();
    set_plist_xattr(
        &path,
        "com.apple.metadata:kMDItemFinderComment",
        Value::String("Reviewed and approved".into()),
    );

    // Time Machine exclusion (com_apple_backup_excludeItem).
    let path = dir.join("time-machine-excluded.txt");
    std::fs::write(&path, "excluded from backup\n").unwrap();
    set_plist_xattr(
        &path,
        "com.apple.metadata:com_apple_backup_excludeItem",
        Value::String("com.apple.backupd".into()),
    );

    // Plain (non-plist) extended attribute.
    let path = dir.join("with-xattr.txt");
    std::fs::write(&path, "a file with a plain xattr\n").unwrap();
    xattr::set(&path, "com.example.test", b"hello world").unwrap();

    // Quarantine (com.apple.quarantine): flags;timestamp;agent;uuid.
    let path = dir.join("quarantined.txt");
    std::fs::write(&path, "downloaded and quarantined\n").unwrap();
    xattr::set(
        &path,
        "com.apple.quarantine",
        b"0083;5e8c5b22;Safari;F8E9B7C8-1234-5678-9ABC-DEF012345678",
    )
    .unwrap();

    // Finder tags (_kMDItemUserTags): "name\ncolor-index" or plain name.
    let path = dir.join("tagged.txt");
    std::fs::write(&path, "a file with Finder tags\n").unwrap();
    set_plist_xattr(
        &path,
        "com.apple.metadata:_kMDItemUserTags",
        Value::Array(vec![
            Value::String("Important\n6".into()),
            Value::String("Work".into()),
        ]),
    );

    // Hidden file flag (UF_HIDDEN, `chflags hidden`).
    let path = dir.join("hidden.txt");
    std::fs::write(&path, "hidden from Finder\n").unwrap();
    let c_path = {
        use std::os::unix::ffi::OsStrExt;
        std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap()
    };
    let rc = unsafe { libc::chflags(c_path.as_ptr(), libc::UF_HIDDEN) };
    assert_eq!(rc, 0, "chflags(UF_HIDDEN) failed");
}

fn run(path: &Path) -> String {
    let output = Command::cargo_bin("dtls")
        .unwrap()
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("stdout was not utf-8")
}

#[test]
fn hello_txt_size_and_sha256() {
    let out = run(&fixture("hello.txt"));
    assert!(out.contains("Size:        6 B (6 bytes)"), "{out}");
    assert!(
        out.contains("5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"),
        "{out}"
    );
}

#[test]
fn file_name_and_absolute_path() {
    let path = fixture("hello.txt");
    let out = run(&path);
    assert!(out.contains("hello.txt"), "{out}");
    assert!(
        out.contains(&format!("({})", path.canonicalize().unwrap().display())),
        "{out}"
    );
}

#[test]
fn utf8_text_detection() {
    let out = run(&fixture("utf8.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    UTF-8"), "{out}");
}

#[test]
fn latin1_detected_as_windows_1252() {
    let out = run(&fixture("latin1.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    windows-1252"), "{out}");
}

#[test]
fn utf16_without_bom_classified_as_binary() {
    let out = run(&fixture("utf16.txt"));
    assert!(out.contains("Type:        binary"), "{out}");
    assert!(!out.contains("Encoding:"), "{out}");
}

#[test]
fn utf16_with_bom_detected_as_text() {
    let out = run(&fixture("utf16le-bom.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    UTF-16LE"), "{out}");
}

#[test]
fn utf32_be_with_bom_detected_as_text() {
    // Without a BOM check, UTF-32BE's leading 00 00 FE FF would flag NUL → binary.
    let out = run(&fixture("utf32be-bom.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    UTF-32BE"), "{out}");
}

#[test]
fn binary_without_magic_is_binary() {
    let out = run(&fixture("binary.bin"));
    assert!(out.contains("Type:        binary"), "{out}");
    assert!(!out.contains("Encoding:"), "{out}");
}

#[test]
fn png_recognised_by_magic() {
    let out = run(&fixture("tiny.png"));
    assert!(out.contains("image/png"), "{out}");
    assert!(!out.contains("Type:        binary"), "{out}");
    assert!(!out.contains("Encoding:"), "{out}");
}

#[test]
fn jpeg_exif_tags_are_listed() {
    let out = run(&fixture("with-exif.jpg"));
    assert!(out.contains("image/jpeg"), "{out}");
    assert!(out.contains("EXIF:"), "{out}");
    assert!(out.contains("Make = \"Acme Cameras\""), "{out}");
    assert!(out.contains("Model = \"Model X\""), "{out}");
    assert!(out.contains("ExposureTime = 1/250 s"), "{out}");
    assert!(out.contains("FNumber = f/2.8"), "{out}");
    assert!(out.contains("FocalLength = 50 mm"), "{out}");
}

#[test]
fn png_without_exif_omits_exif_section() {
    let out = run(&fixture("tiny.png"));
    assert!(!out.contains("EXIF:"), "{out}");
}

#[test]
fn regular_file_omits_symlink_and_xattr_sections() {
    let out = run(&fixture("hello.txt"));
    assert!(!out.contains("Symlink:"), "{out}");
    assert!(!out.contains("Extended attributes:"), "{out}");
}

#[test]
fn permissions_and_owner_lines_present() {
    let out = run(&fixture("hello.txt"));
    assert!(out.contains("Permissions: "), "{out}");
    assert!(out.contains("Owner:       "), "{out}");
    assert!(out.contains("Inode:       "), "{out}");
    assert!(!out.contains("Hard links:"), "{out}");
}

#[test]
fn human_size_scales_to_kib() {
    let out = run(&fixture("two-kib.bin"));
    assert!(out.contains("Size:        2.00 KiB (2048 bytes)"), "{out}");
}

#[test]
fn symlink_shows_target() {
    let out = run(&fixture("working-link.txt"));
    assert!(out.contains("Symlink:     -> link-target.txt"), "{out}");
    assert!(!out.contains("(target does not exist)"), "{out}");
}

#[test]
fn broken_symlink_marks_target_missing() {
    let out = run(&fixture("broken-link.txt"));
    assert!(out.contains("Symlink:     -> nonexistent-target"), "{out}");
    assert!(out.contains("(target does not exist)"), "{out}");
    assert!(out.contains("(symlink target unreachable)"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn plain_xattr_is_listed() {
    let out = run(&darwin_fixture("with-xattr.txt"));
    assert!(out.contains("Extended attributes:"), "{out}");
    assert!(out.contains("com.example.test"), "{out}");
    assert!(out.contains("hello world"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn quarantine_xattr_is_decoded() {
    let out = run(&darwin_fixture("quarantined.txt"));
    assert!(out.contains("quarantine:"), "{out}");
    assert!(out.contains("Safari"), "{out}");
    assert!(out.contains("flags=0083"), "{out}");
    assert!(
        out.contains("F8E9B7C8-1234-5678-9ABC-DEF012345678"),
        "{out}"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn finder_tags_xattr_is_decoded() {
    let out = run(&darwin_fixture("tagged.txt"));
    assert!(out.contains("Finder tags:"), "{out}");
    assert!(out.contains("Important (red)"), "{out}");
    assert!(out.contains("Work"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn where_from_xattr_is_decoded() {
    let out = run(&darwin_fixture("downloaded-file.txt"));
    assert!(out.contains("Where from:"), "{out}");
    assert!(out.contains("https://example.com/downloaded.zip"), "{out}");
    assert!(out.contains("https://example.com/page.html"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn finder_comment_xattr_is_decoded() {
    let out = run(&darwin_fixture("commented-file.txt"));
    assert!(out.contains("Finder comment:"), "{out}");
    assert!(out.contains("Reviewed and approved"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn time_machine_exclusion_xattr_is_decoded() {
    let out = run(&darwin_fixture("time-machine-excluded.txt"));
    assert!(out.contains("Time Machine:"), "{out}");
    assert!(out.contains("com.apple.backupd"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn hidden_flag_is_reported() {
    let out = run(&darwin_fixture("hidden.txt"));
    assert!(out.contains("Flags:       hidden"), "{out}");
}
