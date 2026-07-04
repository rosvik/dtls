//! Tests for src/features/size.rs: human-readable size formatting.

mod common;

use common::{fixture, run};

#[test]
fn small_file_size_in_bytes() {
    let out = run(&fixture("hello.txt"));
    assert!(out.contains("Size:        6 B (6 bytes)"), "{out}");
}

#[test]
fn human_size_scales_to_kib() {
    let out = run(&fixture("two-kib.bin"));
    assert!(out.contains("Size:        2.00 KiB (2048 bytes)"), "{out}");
}
