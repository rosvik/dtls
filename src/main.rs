mod context;
mod features;
mod output;

use anyhow::Result;
use clap::Parser;
use clio::ClioPath;
use colored::Colorize;

use crate::context::Context;
use crate::features::{dates, exif, hash, kind, permissions, size, symlink, xattrs};

#[derive(Parser, Debug)]
#[command(
    name = "dtls",
    version,
    about = "Print detailed information about a file",
    disable_version_flag = true
)]
struct Args {
    /// File to inspect
    file: ClioPath,

    /// Print version
    #[arg(short = 'v', long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let ctx = Context::new(args.file.path())?;

    print_header(&ctx);
    kind::print(&ctx);
    size::print(&ctx);
    permissions::print(&ctx);
    #[cfg(target_os = "macos")]
    features::flags::print(&ctx);
    dates::print(&ctx);
    symlink::print(&ctx)?;
    hash::print(&ctx)?;
    xattrs::print(&ctx)?;
    exif::print(&ctx);

    Ok(())
}

fn print_header(ctx: &Context) {
    let name = ctx
        .path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| ctx.path.display().to_string());
    let abs_path = std::path::absolute(&ctx.path).unwrap_or_else(|_| ctx.path.clone());
    println!(
        "{} {}{}{}",
        name.bold().green(),
        "(".dimmed(),
        abs_path.display().to_string().dimmed(),
        ")".dimmed()
    );
    let separator_len = name.len() + abs_path.display().to_string().len() + 3;
    let separator_len = ctx.terminal.width.min(separator_len);
    println!("{}", "─".repeat(separator_len));
}
