use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;

use crate::progress::Progress;
use crate::puzzle::{self, Puzzle};
use crate::ui;

pub fn run() -> Result<()> {
    let root = puzzle::find_project_root()?;
    let puzzles = puzzle::discover(&root)?;
    let progress = Progress::load()?;

    let mut by_module: BTreeMap<String, Vec<&Puzzle>> = BTreeMap::new();
    for p in &puzzles {
        by_module.entry(p.meta.module.clone()).or_default().push(p);
    }

    let mut unlocked = true;
    let color = ui::use_color();
    for (module, ps) in &by_module {
        println!(
            "\n{}",
            if color {
                module.bold().to_string()
            } else {
                module.clone()
            }
        );
        for p in ps {
            let solved = progress.is_solved(&p.meta.id);
            let marker = if solved {
                if color {
                    "✓".green().to_string()
                } else {
                    "[x]".to_string()
                }
            } else if unlocked {
                if color {
                    "·".yellow().to_string()
                } else {
                    "[ ]".to_string()
                }
            } else if color {
                "🔒".to_string()
            } else {
                "[L]".to_string()
            };
            println!("  {marker}  {}  {}", p.meta.id, p.meta.title);
            if !solved {
                unlocked = false;
            }
        }
    }
    Ok(())
}
