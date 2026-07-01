use colored::Colorize;

use crate::context::Context;
use crate::format::label;

pub fn print(ctx: &Context) {
    match &ctx.target_meta {
        Some(m) => println!(
            "{}{} {}",
            label("Size:"),
            human_size(m.len()),
            format!("({} bytes)", m.len()).dimmed()
        ),
        None => println!(
            "{}{}",
            label("Size:"),
            "(symlink target unreachable)".dimmed()
        ),
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", value, UNITS[unit])
}
