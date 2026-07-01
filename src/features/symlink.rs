use std::fs;

use colored::Colorize;

use crate::context::Context;
use crate::output::format::label;

pub fn print(ctx: &Context) {
    if !ctx.is_symlink {
        return;
    }
    let Ok(target) = fs::read_link(&ctx.path) else {
        return;
    };
    println!(
        "{}-> {}",
        label("Symlink:"),
        target.display().to_string().cyan()
    );
    if ctx.target_meta.is_none() {
        println!("             {}", "(target does not exist)".red());
    }
}
