use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::puzzle::Puzzle;

pub fn home_dir() -> Result<PathBuf> {
    let base = dirs::home_dir().ok_or_else(|| anyhow!("could not resolve home directory"))?;
    Ok(base.join(".bashlings"))
}

pub fn workspaces_dir() -> Result<PathBuf> {
    let p = home_dir()?.join("workspaces");
    std::fs::create_dir_all(&p)?;
    Ok(p)
}

pub fn progress_path() -> Result<PathBuf> {
    let dir = home_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("progress.json"))
}

pub fn current_path() -> Result<PathBuf> {
    let dir = home_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("current.json"))
}

pub fn new_run_id() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Copy a puzzle's template directory into a fresh workspace.
pub fn copy_template(puzzle: &Puzzle, run_id: &str) -> Result<PathBuf> {
    let target = workspaces_dir()?.join(run_id);
    if target.exists() {
        return Err(anyhow!("workspace already exists: {}", target.display()));
    }
    std::fs::create_dir_all(&target)?;
    copy_dir(&puzzle.dir, &target).with_context(|| {
        format!(
            "copying puzzle {} into {}",
            puzzle.meta.id,
            target.display()
        )
    })?;
    Ok(target)
}

/// Recursive copy of dir contents (not the dir itself) into dst.
fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    let mut opts = fs_extra::dir::CopyOptions::new();
    opts.copy_inside = true;
    opts.content_only = true;
    fs_extra::dir::copy(src, dst, &opts)
        .map(|_| ())
        .map_err(|e| anyhow!("copy failed: {e}"))
}

pub fn wipe(dir: &Path) -> Result<()> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).with_context(|| format!("removing {}", dir.display()))?;
    }
    Ok(())
}
