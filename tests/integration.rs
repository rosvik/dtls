mod common;

use common::{fixture, run};

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
