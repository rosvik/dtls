use std::fs::Metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use colored::Colorize;

use crate::format::label;

pub fn print(meta: &Metadata) {
    let mode = meta.permissions().mode();
    println!(
        "{}{} {}",
        label("Permissions:"),
        format_mode(mode),
        format!("({:04o})", mode & 0o7777).dimmed()
    );

    let uid = meta.uid();
    let user = uzers::get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "?".to_string());
    let gid = meta.gid();
    let group = uzers::get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "?".to_string());
    println!(
        "{}{}:{} {}",
        label("Owner:"),
        user,
        group,
        format!("({}:{})", uid, gid).dimmed()
    );

    println!("{}{}", label("Inode:"), meta.ino());
    if meta.nlink() > 1 {
        println!("{}{}", label("Hard links:"), meta.nlink());
    }
}

fn format_mode(mode: u32) -> String {
    let mut s = String::with_capacity(9 * 12);
    for (bit, ch) in [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ] {
        let part = if mode & bit != 0 {
            match ch {
                'r' => ch.to_string().yellow(),
                'w' => ch.to_string().red(),
                'x' => ch.to_string().green(),
                _ => ch.to_string().normal(),
            }
        } else {
            "-".dimmed()
        };
        s.push_str(&part.to_string());
    }
    s
}
