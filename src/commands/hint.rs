use anyhow::{Result, anyhow};

use crate::progress::Current;
use crate::puzzle;

pub fn run() -> Result<()> {
    let mut cur = Current::load()?.ok_or_else(|| anyhow!("no puzzle in progress"))?;
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let p = puzzles
        .iter()
        .find(|p| p.meta.id == cur.puzzle_id)
        .ok_or_else(|| anyhow!("puzzle {} no longer exists", cur.puzzle_id))?;

    let idx = cur.hints_shown as usize;
    if idx >= p.meta.hints.len() {
        println!("No more hints — you have all {}.", p.meta.hints.len());
        return Ok(());
    }
    println!("Hint {} of {}:", idx + 1, p.meta.hints.len());
    println!("  {}", p.meta.hints[idx]);
    cur.hints_shown += 1;
    cur.save()?;
    Ok(())
}
