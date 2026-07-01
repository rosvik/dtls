use std::fs::Metadata;
use std::time::SystemTime;

use chrono::{DateTime, Local};

use crate::format::print_label_value_pairs;

pub fn print(metadata: &Metadata, terminal_width: usize) {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Ok(t) = metadata.created() {
        pairs.push(("Created:", fmt_time(t)));
    }
    if let Ok(t) = metadata.modified() {
        pairs.push(("Modified:", fmt_time(t)));
    }
    if let Ok(t) = metadata.accessed() {
        pairs.push(("Accessed:", fmt_time(t)));
    }
    print_label_value_pairs(&pairs, terminal_width);
}

fn fmt_time(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%Y-%m-%d %H:%M:%S %z").to_string()
}
