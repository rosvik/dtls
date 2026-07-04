//! Tests for src/features/kind.rs: file type detection (magic bytes,
//! text/binary classification, text encoding).

mod common;

use common::{fixture, run};

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
