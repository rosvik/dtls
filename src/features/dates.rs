use std::time::SystemTime;

use chrono::{DateTime, Local};

use crate::context::Context;
use crate::format::print_label_value_pairs;

pub fn print(ctx: &Context) {
    let Some(metadata) = &ctx.target_meta else {
        return;
    };
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
    print_label_value_pairs(&pairs, ctx.terminal.width);
}

fn fmt_time(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%Y-%m-%d %H:%M:%S %z").to_string()
}
