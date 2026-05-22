use anyhow::Result;
use is_terminal::IsTerminal;
use std::io::{BufRead, Write};

pub fn use_color() -> bool {
    std::io::stdout().is_terminal()
}

pub fn stdin_is_tty() -> bool {
    std::io::stdin().is_terminal()
}

/// Print a y/N prompt and return true on yes. Non-tty stdin defaults to false.
pub fn confirm(prompt: &str) -> Result<bool> {
    if !stdin_is_tty() {
        return Ok(false);
    }
    let mut out = std::io::stdout().lock();
    write!(out, "{prompt} [y/N] ")?;
    out.flush()?;
    drop(out);

    let mut line = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_line(&mut line)?;
    let line = line.trim().to_lowercase();
    Ok(matches!(line.as_str(), "y" | "yes"))
}
