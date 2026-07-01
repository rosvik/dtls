use chrono::{DateTime, Local};
use colored::Colorize;

use crate::context::Context;

pub fn print(ctx: &Context) {
    let Ok(list) = xattr::list(&ctx.path) else {
        return;
    };
    let xattrs: Vec<_> = list.collect();
    if xattrs.is_empty() {
        return;
    }
    println!("{}", "Extended attributes:".bold().cyan());
    for attr in xattrs {
        let attr_name = attr.to_string_lossy();
        match xattr::get(&ctx.path, &attr) {
            Ok(Some(value)) => println!(
                "  {} = {}",
                attr_name.magenta(),
                decode_xattr(&attr_name, &value)
            ),
            Ok(None) => println!("  {} = {}", attr_name.magenta(), "(empty)".dimmed()),
            Err(_) => {}
        }
    }
}

fn decode_xattr(name: &str, value: &[u8]) -> String {
    match name {
        "com.apple.metadata:_kMDItemUserTags" => {
            decode_finder_tags(value).unwrap_or_else(|| display_xattr_value(value))
        }
        "com.apple.quarantine" => {
            decode_quarantine(value).unwrap_or_else(|| display_xattr_value(value))
        }
        _ => display_xattr_value(value),
    }
}

fn decode_finder_tags(value: &[u8]) -> Option<String> {
    let parsed: plist::Value = plist::from_bytes(value).ok()?;
    let arr = parsed.as_array()?;
    let tags: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_string())
        .map(|s| {
            let (name, color) = s.split_once('\n').unwrap_or((s, ""));
            let color_name = match color {
                "1" => Some("gray"),
                "2" => Some("green"),
                "3" => Some("purple"),
                "4" => Some("blue"),
                "5" => Some("yellow"),
                "6" => Some("red"),
                "7" => Some("orange"),
                _ => None,
            };
            match color_name {
                Some(c) => format!("{} ({})", name, c),
                None => name.to_string(),
            }
        })
        .collect();
    if tags.is_empty() {
        return None;
    }
    Some(format!("{} [{}]", "Finder tags:".bold(), tags.join(", ")))
}

fn decode_quarantine(value: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(value).ok()?;
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() < 3 {
        return None;
    }
    let flags = parts[0];
    let timestamp = u64::from_str_radix(parts[1], 16).ok()?;
    let agent = parts[2];
    let event = parts.get(3).copied().unwrap_or("");
    let dt: DateTime<Local> = DateTime::from_timestamp(timestamp as i64, 0)?.with_timezone(&Local);
    let mut out = format!(
        "{} flags={} at={} by={}",
        "quarantine:".yellow().bold(),
        flags,
        dt.format("%Y-%m-%d %H:%M:%S %z"),
        agent.yellow()
    );
    if !event.is_empty() {
        out.push_str(&format!(" event={}", event.dimmed()));
    }
    Some(out)
}

fn display_xattr_value(v: &[u8]) -> String {
    let printable = std::str::from_utf8(v).ok().filter(|s| {
        s.chars()
            .all(|c| !c.is_control() || c == '\n' || c == '\t' || c == '\r')
    });
    match printable {
        Some(s) => format!("{} ({} bytes)", s, v.len()),
        None => {
            let preview: String = v.iter().take(32).map(|b| format!("{:02x}", b)).collect();
            if v.len() > 32 {
                format!("0x{}... ({} bytes)", preview, v.len())
            } else {
                format!("0x{} ({} bytes)", preview, v.len())
            }
        }
    }
}
