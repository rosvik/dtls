//! Tests for src/features/hash.rs: SHA-256 hash calculation.

mod common;

use common::{fixture, run};

#[test]
fn sha256_of_hello_txt() {
    let out = run(&fixture("hello.txt"));
    assert!(
        out.contains("5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"),
        "{out}"
    );
}
