use std::fs;
use std::io::Read;
use std::path::Path;

use colored::Colorize;

use crate::context::Context;
use crate::format::label;

pub fn print(ctx: &Context) {
    let Some(meta) = &ctx.target_meta else {
        return;
    };
    if meta.is_file() {
        if let Ok(Some(t)) = infer::get_from_path(&ctx.path) {
            println!(
                "{}{} ({})",
                label("Type:"),
                t.mime_type(),
                t.extension().dimmed()
            );
        } else if let Ok(kind) = detect_text_kind(&ctx.path) {
            match kind {
                TextKind::Binary => println!("{}{}", label("Type:"), "binary".red()),
                TextKind::Text(enc) => {
                    println!("{}{}", label("Type:"), "text".green());
                    println!("{}{}", label("Encoding:"), enc);
                }
            }
        }
    } else if meta.is_dir() {
        println!("{}{}", label("Type:"), "directory".blue());
    }
}

enum TextKind {
    Binary,
    Text(String),
}

fn detect_bom(bytes: &[u8]) -> Option<&'static str> {
    // UTF-32 BOMs must be checked before UTF-16: UTF-32LE starts with FF FE 00 00
    // which shares its first two bytes with UTF-16LE.
    if bytes.starts_with(b"\x00\x00\xFE\xFF") {
        Some("UTF-32BE")
    } else if bytes.starts_with(b"\xFF\xFE\x00\x00") {
        Some("UTF-32LE")
    } else if bytes.starts_with(b"\xFE\xFF") {
        Some("UTF-16BE")
    } else if bytes.starts_with(b"\xFF\xFE") {
        Some("UTF-16LE")
    } else if bytes.starts_with(b"\xEF\xBB\xBF") {
        Some("UTF-8")
    } else {
        None
    }
}

fn detect_text_kind(path: &Path) -> std::io::Result<TextKind> {
    let mut file = fs::File::open(path)?;
    let mut sample = vec![0u8; 8192];
    let n = file.read(&mut sample)?;
    sample.truncate(n);

    if let Some(enc) = detect_bom(&sample) {
        return Ok(TextKind::Text(enc.to_string()));
    }

    if sample.contains(&0) {
        return Ok(TextKind::Binary);
    }

    let utf8_ok = match std::str::from_utf8(&sample) {
        Ok(_) => true,
        Err(e) => e.error_len().is_none() && sample.len() - e.valid_up_to() <= 3,
    };
    if utf8_ok {
        return Ok(TextKind::Text("UTF-8".to_string()));
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&sample, true);
    let enc = detector.guess(None, true);
    Ok(TextKind::Text(enc.name().to_string()))
}
