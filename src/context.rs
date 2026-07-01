use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::terminal::{Size, terminal_size};

/// Everything gathered once per invocation, read by the feature modules.
pub struct Context {
    pub path: PathBuf,
    pub is_symlink: bool,
    /// Metadata with symlinks followed. `None` means the path is a symlink
    /// whose target is unreachable (a missing path errors before this).
    pub target_meta: Option<Metadata>,
    pub terminal: Size,
}

impl Context {
    pub fn new(path: &Path) -> Result<Self> {
        let symlink_meta = fs::symlink_metadata(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            is_symlink: symlink_meta.file_type().is_symlink(),
            target_meta: fs::metadata(path).ok(),
            terminal: terminal_size(),
        })
    }
}
