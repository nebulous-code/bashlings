use anyhow::{Context, Result, anyhow, bail};
use owo_colors::OwoColorize;

use crate::container;
use crate::progress::{Current, Progress};
use crate::puzzle::{self, Puzzle};
use crate::ui;
use crate::workspace;

pub fn run(puzzle_id: &str, _verbose: bool) -> Result<()> {
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let p = puzzles
        .iter()
        .find(|p| p.meta.id == puzzle_id)
        .ok_or_else(|| anyhow!("unknown puzzle: {puzzle_id}"))?;

    if p.meta.requires.network {
        let prompt = format!("⚠ Puzzle {} requests network access. Continue?", p.meta.id);
        if !ui::confirm(&prompt)? {
            bail!("aborted by user");
        }
    }

    let run_id = workspace::new_run_id();
    let ws = workspace::copy_template(p, &run_id).context("creating workspace")?;

    let cur = Current {
        puzzle_id: p.meta.id.clone(),
        run_id: run_id.clone(),
        workspace: ws.clone(),
        hints_shown: 0,
    };
    cur.save()?;
    let mut progress = Progress::load()?;
    progress.record_attempt(&p.meta.id);
    progress.save()?;

    print_intro(p)?;
    run_workspace(&ws, p)
}

pub fn run_workspace(ws: &std::path::Path, p: &Puzzle) -> Result<()> {
    // setup.sh
    tracing::debug!("running setup.sh");
    let setup = container::run_setup(ws, p.meta.requires.network)?;
    if setup.exit != 0 {
        bail!(
            "setup.sh failed (exit {}). stderr:\n{}",
            setup.exit,
            setup.stderr
        );
    }

    println!("\nDropping into the puzzle shell. Type `exit` when you're done.\n");
    let _ = container::run_interactive_shell(ws, p.meta.requires.network)?;

    println!("\nRunning check.sh...");
    let check = container::run_check(ws, p.meta.requires.network)?;
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
        let msg = format!(
            "Not yet (check exited {}). Try `bashlings retry`, or `bashlings hint`.",
            check.exit
        );
        println!("{}", if color { msg.yellow().to_string() } else { msg });
    }
    Ok(())
}

fn print_intro(p: &Puzzle) -> Result<()> {
    let color = ui::use_color();
    let title = format!("== {} — {} ==", p.meta.id, p.meta.title);
    println!(
        "{}",
        if color {
            title.bold().to_string()
        } else {
            title
        }
    );
    let readme = p.read_readme()?;
    println!("\n{readme}");
    Ok(())
}
