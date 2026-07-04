//! Tests for src/features/symlink.rs: symlink target display and
//! broken-link handling.

mod common;

use common::{fixture, run};

#[test]
fn regular_file_omits_symlink_section() {
    let out = run(&fixture("hello.txt"));
    assert!(!out.contains("Symlink:"), "{out}");
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
