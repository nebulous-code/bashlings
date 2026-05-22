use anyhow::{Result, anyhow, bail};

use crate::progress::Progress;
use crate::puzzle;
use crate::ui;

pub fn run(puzzle_id: &str) -> Result<()> {
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let p = puzzles
        .iter()
        .find(|p| p.meta.id == puzzle_id)
        .ok_or_else(|| anyhow!("unknown puzzle: {puzzle_id}"))?;
    let progress = Progress::load()?;

    if !progress.is_solved(puzzle_id) {
        let prompt = "This will reveal the answer. Continue?";
        if !ui::confirm(prompt)? {
            bail!("aborted");
        }
    }
    let text = p.read_solution()?;
    println!("--- {} solution ---", p.meta.id);
    println!("{}", text.trim_end());
    Ok(())
}
