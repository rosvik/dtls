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
        let decoded = match xattr::get(&ctx.path, &attr) {
            Ok(Some(value)) => decode_xattr(&attr_name, &value),
            Ok(None) => Decoded {
                comment: None,
                value: "(empty)".dimmed().to_string(),
            },
            Err(_) => continue,
        };
        if let Some(comment) = decoded.comment {
            println!("  {}", format!("// {comment}").dimmed());
        }
        println!("  {} = {}", attr_name.magenta(), decoded.value);
    }
}

/// A decoded xattr ready to print: the value shown after `= `, plus an optional
/// Spotlight-schema description rendered as a `// name / description / keywords`
/// comment on the line above it.
struct Decoded {
    comment: Option<String>,
    value: String,
}

fn decode_xattr(name: &str, value: &[u8]) -> Decoded {
    let plain = |value| Decoded {
        comment: None,
        value,
    };
    match name {
        "com.apple.metadata:_kMDItemUserTags" => {
            plain(decode_finder_tags(value).unwrap_or_else(|| display_xattr_value(value)))
        }
        "com.apple.quarantine" => {
            plain(decode_quarantine(value).unwrap_or_else(|| display_xattr_value(value)))
        }
        n if n.starts_with("com.apple.metadata:") => decode_metadata(n, value),
        _ => plain(display_xattr_value(value)),
    }
}

/// Any `com.apple.metadata:<key>` attribute holds the value of the Spotlight
/// attribute `<key>` as a binary plist. Decodes the value generically and, when
/// the key is in the vendored `mdimport -A` schema, attaches its display name,
/// description, and keywords as a comment.
fn decode_metadata(name: &str, value: &[u8]) -> Decoded {
    let key = name.strip_prefix("com.apple.metadata:").unwrap_or(name);
    Decoded {
        comment: schema_comment(key),
        value: render_metadata_value(value).unwrap_or_else(|| display_xattr_value(value)),
    }
}

fn render_metadata_value(value: &[u8]) -> Option<String> {
    let parsed: plist::Value = plist::from_bytes(value).ok()?;
    render_plist_value(&parsed)
}

/// Looks `key` up in the Spotlight schema and joins its human-readable fields
/// (display name, description, keywords — whichever are present) into a single
/// `name / description / keywords` string. Returns `None` for keys the schema
/// doesn't cover.
fn schema_comment(key: &str) -> Option<String> {
    let lookup = |k| {
        mditem_names::SCHEMA
            .binary_search_by_key(&k, |&(k, ..)| k)
            .ok()
            .map(|i| mditem_names::SCHEMA[i])
    };
    // A leading underscore marks a key as private API; the schema often
    // lists the public counterpart (e.g. _kMDItemUserTags → kMDItemUserTags).
    let (_, name, description, keywords) =
        lookup(key).or_else(|| lookup(key.strip_prefix('_')?))?;
    let fields: Vec<&str> = [name, description, keywords]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    (!fields.is_empty()).then(|| fields.join(" / "))
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
                "1" => Some(format!("{}", "gray".dimmed())),
                "2" => Some(format!("{}", "green".green())),
                "3" => Some(format!("{}", "purple".purple())),
                "4" => Some(format!("{}", "blue".blue())),
                "5" => Some(format!("{}", "yellow".bright_yellow())),
                "6" => Some(format!("{}", "red".red())),
                "7" => Some(format!("{}", "orange".yellow())), // Don't @ me
                _ => None,
            };
            match color_name {
                Some(c) => format!("{} ({})", name, c.bold()),
                None => name.to_string(),
            }
        })
        .collect();
    if tags.is_empty() {
        return None;
    }
    Some(tags.join(", "))
}

/// Decodes the `com.apple.quarantine` xattr. It is a string on the format:
/// `flags;hex-unix-time;agent;event-uuid`
///
/// <https://eclecticlight.co/2017/12/11/xattr-com-apple-quarantine-the-quarantine-flag/>
fn decode_quarantine(value: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(value).ok()?;
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() < 3 {
        return None;
    }
    let flags = parts[0];
    let timestamp = u64::from_str_radix(parts[1], 16).ok()?;
    let agent = parts[2];
    let event_uuid = parts.get(3).copied().unwrap_or("");
    let dt: DateTime<Local> = DateTime::from_timestamp(timestamp as i64, 0)?.with_timezone(&Local);
    let mut out = format!(
        "flags={} at={} by={}",
        flags,
        dt.format("%Y-%m-%d %H:%M:%S %z"),
        agent
    );
    if !event_uuid.is_empty() {
        out.push_str(&format!(" event={}", event_uuid.dimmed()));
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
