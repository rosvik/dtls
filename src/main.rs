mod features;
mod format;
mod terminal;

use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use clio::ClioPath;
use colored::Colorize;

use crate::features::{dates, exif, hash, kind, permissions, size, symlink, xattrs};
use crate::terminal::terminal_size;

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
    let path: &Path = args.file.path();

    let symlink_meta = fs::symlink_metadata(path)?;
    let is_symlink = symlink_meta.file_type().is_symlink();
    let target_meta = fs::metadata(path).ok();
    let terminal_size = terminal_size();

    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let abs_path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    println!(
        "{} {}{}{}",
        name.bold().green(),
        "(".dimmed(),
        abs_path.display().to_string().dimmed(),
        ")".dimmed()
    );
    let separator_len = name.len() + abs_path.display().to_string().len() + 3;
    let separator_len = terminal_size.width.min(separator_len);
    println!("{}", "─".repeat(separator_len));

    if let Some(m) = &target_meta {
        kind::print(path, m);
    }
    size::print(target_meta.as_ref());
    if let Some(m) = &target_meta {
        permissions::print(m);
        #[cfg(target_os = "macos")]
        features::flags::print(m);
        dates::print(m, terminal_size.width);
    }
    if is_symlink {
        symlink::print(path, target_meta.is_some())?;
    }
    if target_meta.as_ref().is_some_and(|m| m.is_file()) {
        hash::print(path)?;
    }
    xattrs::print(path)?;
    exif::print(path);

    Ok(())
}
