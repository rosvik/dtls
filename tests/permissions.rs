//! Tests for src/features/permissions.rs: mode, owner, inode, and hard
//! link count.

mod common;

use common::{fixture, run};

#[test]
fn permissions_and_owner_lines_present() {
    let out = run(&fixture("hello.txt"));
    assert!(out.contains("Permissions: "), "{out}");
    assert!(out.contains("Owner:       "), "{out}");
    assert!(out.contains("Inode:       "), "{out}");
    assert!(!out.contains("Hard links:"), "{out}");
}
