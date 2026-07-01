use std::fs;
use std::io::{Read, Result};
use std::path::Path;

use colored::Colorize;
use sha2::{Digest, Sha256};

use crate::context::Context;
use crate::output::format::label;

pub fn print(ctx: &Context) {
    if !ctx.target_meta.as_ref().is_some_and(|m| m.is_file()) {
        return;
    }
    if let Ok(hash) = sha256_file(&ctx.path) {
        println!("{}{}", label("SHA256:"), hash.dimmed());
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
