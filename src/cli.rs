use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "bashlings",
    version,
    about = "Learn bash by solving puzzles in disposable containers."
)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build the base container image (`bashlings-base:latest`).
    Init,
    /// Show modules and progress.
    List,
    /// Show overall progress summary.
    Status,
    /// Work on the next unsolved puzzle.
    Next,
    /// Work on a specific puzzle by id.
    Start { puzzle_id: String },
    /// Relaunch a container against the in-progress workspace.
    Retry,
    /// Wipe the workspace, re-copy the template, restart the puzzle.
    Reset { puzzle_id: String },
    /// Show the next progressive hint for the in-progress puzzle.
    Hint,
    /// Reveal the reference solution for a puzzle.
    Solution { puzzle_id: String },
    /// Self-test: run setup, assert check fails, run solution, assert check passes.
    Verify { puzzle_id: Option<String> },
}
