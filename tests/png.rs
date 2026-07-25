//! Tests for src/features/png.rs: IHDR fields for PNG images.

mod common;

use common::{fixture, run};

#[test]
fn truecolor_png_ihdr_is_printed() {
    let out = run(&fixture("tiny.png"));
    assert!(out.contains("PNG:"), "{out}");
    assert!(out.contains("Dimensions = 1 × 1 pixels"), "{out}");
    assert!(out.contains("Bit depth = 8 bits per sample"), "{out}");
    assert!(out.contains("Color type = truecolor (2)"), "{out}");
    assert!(out.contains("Compression = deflate (0)"), "{out}");
    assert!(out.contains("Filter = adaptive (0)"), "{out}");
    assert!(out.contains("Interlace = none (0)"), "{out}");
}

#[test]
fn indexed_interlaced_png_ihdr_is_printed() {
    let out = run(&fixture("indexed-interlaced.png"));
    assert!(out.contains("Dimensions = 4 × 4 pixels"), "{out}");
    assert!(out.contains("Bit depth = 4 bits per sample"), "{out}");
    assert!(out.contains("Color type = indexed (3)"), "{out}");
    assert!(out.contains("Interlace = Adam7 (1)"), "{out}");
}

#[test]
fn non_png_omits_png_section() {
    let out = run(&fixture("with-exif.jpg"));
    assert!(!out.contains("PNG:"), "{out}");
}

#[test]
fn truncated_png_omits_png_section() {
    // Signature plus a partial IHDR: still detected as image/png, but there
    // are no complete header fields to report.
    let out = run(&fixture("truncated.png"));
    assert!(out.contains("image/png"), "{out}");
    assert!(!out.contains("PNG:"), "{out}");
}
