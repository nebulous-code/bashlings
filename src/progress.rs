use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::workspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Solved,
    Attempted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleProgress {
    pub status: Status,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub hints_shown: u32,
    pub first_solved_at: Option<DateTime<Utc>>,
    pub last_attempted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub version: u32,
    #[serde(default)]
    pub puzzles: BTreeMap<String, PuzzleProgress>,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            version: 1,
            puzzles: BTreeMap::new(),
        }
    }
}

impl Progress {
    pub fn load() -> Result<Self> {
        let path = workspace::progress_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        if raw.trim().is_empty() {
            return Ok(Self::default());
        }
        let p: Progress =
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        Ok(p)
    }

    pub fn save(&self) -> Result<()> {
        let path = workspace::progress_path()?;
        atomic_write(&path, &serde_json::to_vec_pretty(self)?)
    }

    pub fn is_solved(&self, id: &str) -> bool {
        matches!(self.puzzles.get(id), Some(p) if matches!(p.status, Status::Solved))
    }

    pub fn record_attempt(&mut self, id: &str) {
        let entry = self
            .puzzles
            .entry(id.to_string())
            .or_insert(PuzzleProgress {
                status: Status::Attempted,
                attempts: 0,
                hints_shown: 0,
                first_solved_at: None,
                last_attempted_at: None,
            });
        entry.attempts += 1;
        entry.last_attempted_at = Some(Utc::now());
    }

    pub fn record_solved(&mut self, id: &str) {
        let now = Utc::now();
        let entry = self
            .puzzles
            .entry(id.to_string())
            .or_insert(PuzzleProgress {
                status: Status::Solved,
                attempts: 0,
                hints_shown: 0,
                first_solved_at: None,
                last_attempted_at: None,
            });
        // Re-solving must not clear first_solved_at.
        if entry.first_solved_at.is_none() {
            entry.first_solved_at = Some(now);
        }
        entry.status = Status::Solved;
        entry.last_attempted_at = Some(now);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Current {
    pub puzzle_id: String,
    pub run_id: String,
    pub workspace: PathBuf,
    #[serde(default)]
    pub hints_shown: u32,
}

impl Current {
    pub fn load() -> Result<Option<Self>> {
        let path = workspace::current_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&path)?;
        if raw.trim().is_empty() {
            return Ok(None);
        }
        let c: Current = serde_json::from_str(&raw)?;
        Ok(Some(c))
    }

    pub fn save(&self) -> Result<()> {
        let path = workspace::current_path()?;
        atomic_write(&path, &serde_json::to_vec_pretty(self)?)
    }

    pub fn clear() -> Result<()> {
        let path = workspace::current_path()?;
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let tmp = path.with_extension("tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}
