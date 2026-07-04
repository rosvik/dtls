//! Helpers shared by the integration test binaries. Each file in tests/ is
//! compiled as its own crate, so helpers a given binary doesn't use would
//! trigger dead_code warnings there.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;

pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Returns a fixture from tests/fixtures/generated/darwin/, (re)generating
/// them once per test run. These files carry macOS xattrs and file flags
/// that git can't store, so they aren't checked in — they're left in place
/// after the run for poking at dtls manually.
#[cfg(target_os = "macos")]
pub fn darwin_fixture(name: &str) -> PathBuf {
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

    // Plain (non-plist) extended attributes: a printable string and a raw
    // binary value.
    let path = dir.join("with-xattr.txt");
    std::fs::write(&path, "a file with a plain xattr\n").unwrap();
    xattr::set(&path, "com.example.test", b"hello world").unwrap();
    xattr::set(&path, "com.example.binary", &[0xde, 0xad, 0xbe, 0xef]).unwrap();

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

    // Downloaded date (kMDItemDownloadedDate): bplist array of dates, no
    // dedicated decoder — exercises the generic com.apple.metadata:* path.
    let path = dir.join("downloaded-date.txt");
    std::fs::write(&path, "downloaded content\n").unwrap();
    set_plist_xattr(
        &path,
        "com.apple.metadata:kMDItemDownloadedDate",
        Value::Array(vec![Value::Date(plist::Date::from(
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_600_000_000),
        ))]),
    );

    // Made-up com.apple.metadata:* key with a nested value — generic path,
    // raw-key label, nested structures render as plist debug output.
    let path = dir.join("generic-metadata.txt");
    std::fs::write(&path, "generic metadata\n").unwrap();
    let mut nested = plist::Dictionary::new();
    nested.insert("version".into(), Value::Integer(2i64.into()));
    nested.insert(
        "labels".into(),
        Value::Array(vec![Value::String("a".into()), Value::String("b".into())]),
    );
    set_plist_xattr(
        &path,
        "com.apple.metadata:kMDItemDtlsTestKey",
        Value::Dictionary(nested),
    );

    // Non-metadata attribute holding a binary plist.
    let path = dir.join("custom-bplist.txt");
    std::fs::write(&path, "custom bplist xattr\n").unwrap();
    set_plist_xattr(
        &path,
        "com.example.bplist",
        Value::Array(vec![
            Value::String("one".into()),
            Value::String("two".into()),
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

pub fn run(path: &Path) -> String {
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
