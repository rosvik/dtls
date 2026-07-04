//! Tests for src/features/flags.rs: BSD file flags (macOS-only).
#![cfg(target_os = "macos")]

mod common;

use common::{darwin_fixture, run};

/// The `hidden` BSD file flag (`UF_HIDDEN` in `st_flags`, set with
/// `chflags hidden` — see `man 2 chflags` and `man 1 chflags`) tells
/// Finder not to show the file. It's a stat-level flag rather than an
/// xattr, but like xattrs it's macOS metadata that git can't store.
#[test]
fn hidden_flag_is_reported() {
    let out = run(&darwin_fixture("hidden.txt"));
    assert!(out.contains("Flags:       hidden"), "{out}");
}
