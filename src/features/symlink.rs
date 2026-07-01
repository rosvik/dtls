use std::fs;

use anyhow::Result;
use colored::Colorize;

use crate::context::Context;
use crate::output::format::label;

pub fn print(ctx: &Context) -> Result<()> {
    if !ctx.is_symlink {
        return Ok(());
    }
    let target = fs::read_link(&ctx.path)?;
    println!(
        "{}-> {}",
        label("Symlink:"),
        target.display().to_string().cyan()
    );
    if ctx.target_meta.is_none() {
        println!("             {}", "(target does not exist)".red());
    }
    Ok(())
}
