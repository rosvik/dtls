use chrono::{DateTime, Local};
use colored::Colorize;

use crate::context::Context;

mod mditem_names;

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
        n if n.starts_with("com.apple.metadata:") => {
            decode_metadata(n, value).unwrap_or_else(|| display_xattr_value(value))
        }
        _ => display_xattr_value(value),
    }
}

/// Any `com.apple.metadata:<key>` attribute holds the value of the Spotlight
/// attribute `<key>` as a binary plist. Decodes it generically, labeled with
/// the key's display name from the vendored `mdimport -A` table (falling
/// back to the raw key).
fn decode_metadata(name: &str, value: &[u8]) -> Option<String> {
    let key = name.strip_prefix("com.apple.metadata:")?;
    let parsed: plist::Value = plist::from_bytes(value).ok()?;
    let rendered = render_plist_value(&parsed)?;
    let label = display_name(key).unwrap_or(key);
    Some(format!("{} {}", format!("{label}:").bold(), rendered))
}

fn display_name(key: &str) -> Option<&'static str> {
    let lookup = |k| {
        mditem_names::DISPLAY_NAMES
            .binary_search_by_key(&k, |&(k, _)| k)
            .ok()
            .map(|i| mditem_names::DISPLAY_NAMES[i].1)
    };
    // A leading underscore marks a key as private API; the schema often
    // lists the public counterpart (e.g. _kMDItemUserTags → kMDItemUserTags).
    lookup(key).or_else(|| lookup(key.strip_prefix('_')?))
}

fn render_plist_value(v: &plist::Value) -> Option<String> {
    if let Some(s) = render_plist_scalar(v) {
        return Some(s);
    }
    match v {
        plist::Value::Array(arr) => {
            if arr.is_empty() {
                return None;
            }
            match arr
                .iter()
                .map(render_plist_scalar)
                .collect::<Option<Vec<_>>>()
            {
                Some(items) => Some(items.join(", ")),
                None => Some(format!("{v:?}")),
            }
        }
        plist::Value::Dictionary(_) => Some(format!("{v:?}")),
        _ => None,
    }
}

fn render_plist_scalar(v: &plist::Value) -> Option<String> {
    match v {
        plist::Value::String(s) => Some(s.clone()),
        plist::Value::Boolean(b) => Some(b.to_string()),
        plist::Value::Integer(i) => Some(i.to_string()),
        plist::Value::Real(r) => Some(r.to_string()),
        plist::Value::Date(d) => {
            let dt: DateTime<Local> = std::time::SystemTime::from(*d).into();
            Some(dt.format("%Y-%m-%d %H:%M:%S %z").to_string())
        }
        _ => None,
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
    if v.starts_with(b"bplist00") {
        let decoded: Option<plist::Value> = plist::from_bytes(v).ok();
        if let Some(rendered) = decoded.as_ref().and_then(render_plist_value) {
            return format!("{} ({} bytes)", rendered, v.len());
        }
    }
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
