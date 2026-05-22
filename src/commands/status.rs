use anyhow::Result;

use crate::progress::{Current, Progress};
use crate::puzzle;

pub fn run() -> Result<()> {
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let progress = Progress::load()?;

    let total = puzzles.len();
    let solved = puzzles
        .iter()
        .filter(|p| progress.is_solved(&p.meta.id))
        .count();
    println!("Progress: {solved}/{total} puzzles solved");

    if let Some(cur) = Current::load()? {
        println!("In progress: {} (run {})", cur.puzzle_id, cur.run_id);
        println!("  workspace: {}", cur.workspace.display());
        println!("  hints shown: {}", cur.hints_shown);
    } else {
        println!("No puzzle in progress.");
    }
    Ok(())
}
