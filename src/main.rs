use anyhow::Result;
use bashlings::cli;
use bashlings::commands;
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let default_level = if args.verbose {
        "bashlings=debug"
    } else {
        "bashlings=warn"
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr)
        .init();

    use cli::Command::*;
    match args.command {
        Init => commands::init::run(args.verbose),
        List => commands::list::run(),
        Status => commands::status::run(),
        Next => commands::next::run(args.verbose),
        Start { puzzle_id } => commands::start::run(&puzzle_id, args.verbose),
        Retry => commands::retry::run(args.verbose),
        Reset { puzzle_id } => commands::reset::run(&puzzle_id, args.verbose),
        Hint => commands::hint::run(),
        Solution { puzzle_id } => commands::solution::run(&puzzle_id),
        Verify { puzzle_id } => commands::verify::run(puzzle_id.as_deref(), args.verbose),
    }
}
