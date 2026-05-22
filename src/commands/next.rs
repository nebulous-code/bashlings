use anyhow::{Result, bail};

use crate::commands::start;
use crate::progress::Progress;
use crate::puzzle;

pub fn run(verbose: bool) -> Result<()> {
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let progress = Progress::load()?;
    let next_id = puzzles
        .iter()
        .find(|p| !progress.is_solved(&p.meta.id))
        .map(|p| p.meta.id.clone());
    match next_id {
        Some(id) => {
            println!("Next puzzle: {id}");
            start::run(&id, verbose)
        }
        None => {
            if puzzles.is_empty() {
                bail!("no puzzles found in project");
            }
            println!("All puzzles solved! 🎉");
            Ok(())
        }
    }
}
