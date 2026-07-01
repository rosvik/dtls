use std::fs;
use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::format::label;

pub fn print(path: &Path, target_reachable: bool) -> Result<()> {
    let target = fs::read_link(path)?;
    println!(
        "{}-> {}",
        label("Symlink:"),
        target.display().to_string().cyan()
    );
    if !target_reachable {
        println!("             {}", "(target does not exist)".red());
    }
    Ok(())
}
