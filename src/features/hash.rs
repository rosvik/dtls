use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use colored::Colorize;
use sha2::{Digest, Sha256};

use crate::format::label;

pub fn print(path: &Path) -> Result<()> {
    println!("{}{}", label("SHA256:"), sha256_file(path)?.dimmed());
    Ok(())
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
