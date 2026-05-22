use anyhow::{Result, anyhow, bail};
use owo_colors::OwoColorize;

use crate::container;
use crate::puzzle::{self, Puzzle};
use crate::ui;
use crate::workspace;

pub fn run(puzzle_id: Option<&str>, verbose: bool) -> Result<()> {
    let root = puzzle::find_project_root()?;
    let all = puzzle::discover(&root)?;
    let targets: Vec<&Puzzle> = match puzzle_id {
        Some(id) => vec![
            all.iter()
                .find(|p| p.meta.id == id)
                .ok_or_else(|| anyhow!("unknown puzzle: {id}"))?,
        ],
        None => all.iter().collect(),
    };
    if targets.is_empty() {
        bail!("no puzzles to verify");
    }

    let mut failures: Vec<(String, String)> = Vec::new();
    for p in &targets {
        match verify_one(p, verbose) {
            Ok(()) => print_result(&p.meta.id, true),
            Err(e) => {
                print_result(&p.meta.id, false);
                failures.push((p.meta.id.clone(), e.to_string()));
            }
        }
    }

    println!(
        "\n{} / {} puzzles passed verification.",
        targets.len() - failures.len(),
        targets.len()
    );
    if !failures.is_empty() {
        println!("\nFailures:");
        for (id, msg) in &failures {
            println!("  - {id}: {msg}");
        }
        bail!("verification failed");
    }
    Ok(())
}

fn print_result(id: &str, ok: bool) {
    let color = ui::use_color();
    if ok {
        let m = "PASS";
        println!(
            "  {}  {id}",
            if color {
                m.green().to_string()
            } else {
                m.to_string()
            }
        );
    } else {
        let m = "FAIL";
        println!(
            "  {}  {id}",
            if color {
                m.red().to_string()
            } else {
                m.to_string()
            }
        );
    }
}

fn verify_one(p: &Puzzle, verbose: bool) -> Result<()> {
    let run_id = format!("verify-{}", workspace::new_run_id());
    let ws = workspace::copy_template(p, &run_id)?;
    let result = (|| -> Result<()> {
        // 1. setup
        let s = container::run_setup(&ws, p.meta.requires.network)?;
        if s.exit != 0 {
            bail!("setup.sh failed (exit {}): {}", s.exit, s.stderr);
        }
        // 2. check should fail
        let c1 = container::run_check(&ws, p.meta.requires.network)?;
        if c1.exit == 0 {
            bail!("check.sh passed before solution.sh ran (initial state is not unsolved)");
        }
        // 3. solution
        let sol = container::run_solution(&ws, p.meta.requires.network)?;
        if sol.exit != 0 {
            bail!("solution.sh failed (exit {}): {}", sol.exit, sol.stderr);
        }
        // 4. check should pass
        let c2 = container::run_check(&ws, p.meta.requires.network)?;
        if c2.exit != 0 {
            bail!(
                "check.sh failed after solution.sh (exit {}): {}",
                c2.exit,
                c2.stderr
            );
        }
        Ok(())
    })();
    // Always wipe the verify workspace.
    let _ = workspace::wipe(&ws);
    if verbose {
        tracing::debug!(puzzle = %p.meta.id, ws = %ws.display(), "verify cleanup done");
    }
    result
}
