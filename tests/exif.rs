//! Tests for src/features/exif.rs: EXIF tag listing for images.

mod common;

use common::{fixture, run};

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
