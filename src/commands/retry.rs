use anyhow::{Result, anyhow, bail};
use owo_colors::OwoColorize;

use crate::container;
use crate::progress::{Current, Progress};
use crate::puzzle;
use crate::ui;

pub fn run(_verbose: bool) -> Result<()> {
    let cur = Current::load()?.ok_or_else(|| {
        anyhow!("no puzzle in progress (run `bashlings start <id>` or `bashlings next`)")
    })?;
    if !cur.workspace.exists() {
        bail!(
            "workspace missing: {} (try `bashlings reset {}`)",
            cur.workspace.display(),
            cur.puzzle_id
        );
    }
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let p = puzzles
        .iter()
        .find(|p| p.meta.id == cur.puzzle_id)
        .ok_or_else(|| anyhow!("puzzle {} no longer exists", cur.puzzle_id))?;

    let mut progress = Progress::load()?;
    progress.record_attempt(&p.meta.id);
    progress.save()?;

    println!("\nRetry — dropping back into your existing workspace.\n");
    let _ = container::run_interactive_shell(&cur.workspace, p.meta.requires.network)?;
    println!("\nRunning check.sh...");
    let check = container::run_check(&cur.workspace, p.meta.requires.network)?;
    if !check.stdout.is_empty() {
        println!("{}", check.stdout.trim_end());
    }
    let color = ui::use_color();
    if check.exit == 0 {
        let msg = "Solved!";
        println!(
            "{}",
            if color {
                msg.green().bold().to_string()
            } else {
                msg.to_string()
            }
        );
        let mut progress = Progress::load()?;
        progress.record_solved(&p.meta.id);
        progress.save()?;
    } else {
        let msg = format!("Not yet (check exited {}).", check.exit);
        println!("{}", if color { msg.yellow().to_string() } else { msg });
    }
    Ok(())
}
