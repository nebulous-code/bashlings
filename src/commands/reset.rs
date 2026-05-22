use anyhow::{Context, Result, anyhow};

use crate::commands::start;
use crate::progress::{Current, Progress};
use crate::puzzle;
use crate::workspace;

pub fn run(puzzle_id: &str, _verbose: bool) -> Result<()> {
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let p = puzzles
        .iter()
        .find(|p| p.meta.id == puzzle_id)
        .ok_or_else(|| anyhow!("unknown puzzle: {puzzle_id}"))?;

    if let Some(cur) = Current::load()?
        && cur.puzzle_id == puzzle_id
    {
        workspace::wipe(&cur.workspace)
            .with_context(|| format!("wiping {}", cur.workspace.display()))?;
        Current::clear()?;
    }

    let run_id = workspace::new_run_id();
    let ws = workspace::copy_template(p, &run_id)?;
    let cur = Current {
        puzzle_id: p.meta.id.clone(),
        run_id,
        workspace: ws.clone(),
        hints_shown: 0,
    };
    cur.save()?;

    let mut progress = Progress::load()?;
    progress.record_attempt(&p.meta.id);
    progress.save()?;

    println!("Reset {puzzle_id}. Fresh workspace at {}", ws.display());
    start::run_workspace(&ws, p)
}
