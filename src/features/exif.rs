use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use colored::Colorize;

use crate::context::Context;

pub fn print(ctx: &Context) {
    let Some(fields) = read_exif(&ctx.path) else {
        return;
    };
    println!("{}", "EXIF:".bold().cyan());
    for field in fields {
        println!(
            "  {} = {}",
            field.tag.magenta(),
            truncate(&field.value, 120)
        );
    }
}

struct ExifField {
    tag: String,
    value: String,
}

fn read_exif(path: &Path) -> Option<Vec<ExifField>> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let fields: Vec<_> = exif
        .fields()
        .map(|f| ExifField {
            tag: f.tag.to_string(),
            value: f.display_value().with_unit(&exif).to_string(),
        })
        .collect();
    (!fields.is_empty()).then_some(fields)
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push('…');
    out
}
