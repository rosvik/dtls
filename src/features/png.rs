use std::fs::File;
use std::io::Read;
use std::path::Path;

use colored::Colorize;

use crate::context::Context;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// IHDR data is a fixed 13 bytes, and the chunk always comes first.
const IHDR_DATA_LEN: usize = 13;
/// Signature + chunk length + chunk type + IHDR data.
const HEADER_LEN: usize = 8 + 4 + 4 + IHDR_DATA_LEN;

pub fn print(ctx: &Context) {
    let Some(ihdr) = read_ihdr(&ctx.path) else {
        return;
    };
    println!("{}", "PNG:".bold().cyan());
    for (name, value) in ihdr.fields() {
        println!("  {} = {}", name.magenta(), value);
    }
}

struct Ihdr {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression: u8,
    filter: u8,
    interlace: u8,
}

fn read_ihdr(path: &Path) -> Option<Ihdr> {
    let mut buf = [0u8; HEADER_LEN];
    File::open(path).ok()?.read_exact(&mut buf).ok()?;
    parse_ihdr(&buf)
}

fn parse_ihdr(buf: &[u8; HEADER_LEN]) -> Option<Ihdr> {
    if buf[..8] != SIGNATURE {
        return None;
    }
    let data_len = u32::from_be_bytes(buf[8..12].try_into().ok()?);
    if data_len as usize != IHDR_DATA_LEN || &buf[12..16] != b"IHDR" {
        return None;
    }
    let data = &buf[16..];
    Some(Ihdr {
        width: u32::from_be_bytes(data[0..4].try_into().ok()?),
        height: u32::from_be_bytes(data[4..8].try_into().ok()?),
        bit_depth: data[8],
        color_type: data[9],
        compression: data[10],
        filter: data[11],
        interlace: data[12],
    })
}

impl Ihdr {
    fn fields(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "Dimensions",
                format!("{} × {} pixels", self.width, self.height),
            ),
            ("Bit depth", format!("{} bits per sample", self.bit_depth)),
            (
                "Color type",
                named(color_type(self.color_type), self.color_type),
            ),
            (
                "Compression",
                named(compression(self.compression), self.compression),
            ),
            ("Filter", named(filter(self.filter), self.filter)),
            (
                "Interlace",
                named(interlace(self.interlace), self.interlace),
            ),
        ]
    }
}

/// Renders a coded IHDR field as `name (code)`, flagging codes the PNG spec
/// doesn't define.
fn named(name: Option<&str>, code: u8) -> String {
    let name = match name {
        Some(name) => name.normal(),
        None => "unknown".red(),
    };
    format!("{} {}", name, format!("({code})").dimmed())
}

fn color_type(code: u8) -> Option<&'static str> {
    match code {
        0 => Some("grayscale"),
        2 => Some("truecolor"),
        3 => Some("indexed"),
        4 => Some("grayscale + alpha"),
        6 => Some("truecolor + alpha"),
        _ => None,
    }
}

fn compression(code: u8) -> Option<&'static str> {
    (code == 0).then_some("deflate")
}

fn filter(code: u8) -> Option<&'static str> {
    (code == 0).then_some("adaptive")
}

fn interlace(code: u8) -> Option<&'static str> {
    match code {
        0 => Some("none"),
        1 => Some("Adam7"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(chunk_len: u32, chunk_type: &[u8; 4], data: [u8; IHDR_DATA_LEN]) -> [u8; HEADER_LEN] {
        let mut buf = [0u8; HEADER_LEN];
        buf[..8].copy_from_slice(&SIGNATURE);
        buf[8..12].copy_from_slice(&chunk_len.to_be_bytes());
        buf[12..16].copy_from_slice(chunk_type);
        buf[16..].copy_from_slice(&data);
        buf
    }

    /// 800 × 600, 8-bit, truecolor + alpha, Adam7 interlaced.
    fn sample_data() -> [u8; IHDR_DATA_LEN] {
        [0, 0, 3, 32, 0, 0, 2, 88, 8, 6, 0, 0, 1]
    }

    #[test]
    fn parses_ihdr_fields() {
        let ihdr = parse_ihdr(&header(13, b"IHDR", sample_data())).unwrap();
        assert_eq!(ihdr.width, 800);
        assert_eq!(ihdr.height, 600);
        assert_eq!(ihdr.bit_depth, 8);
        assert_eq!(ihdr.color_type, 6);
        assert_eq!(ihdr.compression, 0);
        assert_eq!(ihdr.filter, 0);
        assert_eq!(ihdr.interlace, 1);
    }

    #[test]
    fn rejects_wrong_signature() {
        let mut buf = header(13, b"IHDR", sample_data());
        buf[1] = b'X';
        assert!(parse_ihdr(&buf).is_none());
    }

    #[test]
    fn rejects_first_chunk_that_is_not_ihdr() {
        assert!(parse_ihdr(&header(13, b"IDAT", sample_data())).is_none());
    }

    #[test]
    fn rejects_wrong_ihdr_length() {
        assert!(parse_ihdr(&header(12, b"IHDR", sample_data())).is_none());
    }

    #[test]
    fn undefined_codes_are_flagged_as_unknown() {
        let mut data = sample_data();
        data[9] = 7; // no such color type
        let ihdr = parse_ihdr(&header(13, b"IHDR", data)).unwrap();
        let fields = ihdr.fields();
        let color_type = &fields.iter().find(|(n, _)| *n == "Color type").unwrap().1;
        assert_eq!(color_type, "unknown (7)");
    }
}
