use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub id: String,
    pub title: String,
    pub module: String,
    pub difficulty: String,
    #[serde(default)]
    pub estimated_minutes: u32,
    #[serde(default)]
    pub hints: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requires: Requires,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Requires {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub writable_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Puzzle {
    pub meta: Meta,
    pub dir: PathBuf,
}

impl Puzzle {
    pub fn readme_path(&self) -> PathBuf {
        self.dir.join("README.md")
    }
    pub fn solution_path(&self) -> PathBuf {
        self.dir.join("solution.sh")
    }

    pub fn read_readme(&self) -> Result<String> {
        std::fs::read_to_string(self.readme_path())
            .with_context(|| format!("reading README.md for {}", self.meta.id))
    }
    pub fn read_solution(&self) -> Result<String> {
        std::fs::read_to_string(self.solution_path())
            .with_context(|| format!("reading solution.sh for {}", self.meta.id))
    }
}

pub fn load(dir: &Path) -> Result<Puzzle> {
    let meta_path = dir.join("meta.yaml");
    let raw = std::fs::read_to_string(&meta_path)
        .with_context(|| format!("reading {}", meta_path.display()))?;
    let meta: Meta =
        serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", meta_path.display()))?;

    let expected_module = meta.id.split('/').next().unwrap_or("");
    if meta.module != expected_module {
        bail!(
            "{}: meta.module ({}) does not match id prefix ({})",
            meta_path.display(),
            meta.module,
            expected_module
        );
    }

    for script in ["setup.sh", "check.sh", "solution.sh", "README.md"] {
        let p = dir.join(script);
        if !p.exists() {
            bail!("{}: missing required file {script}", dir.display());
        }
    }

    Ok(Puzzle {
        meta,
        dir: dir.to_path_buf(),
    })
}

/// Walk `root/puzzles` and return all puzzles sorted by id.
pub fn discover(root: &Path) -> Result<Vec<Puzzle>> {
    let puzzles_root = root.join("puzzles");
    if !puzzles_root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in WalkDir::new(&puzzles_root).min_depth(1).max_depth(4) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "meta.yaml" {
            continue;
        }
        let dir = entry
            .path()
            .parent()
            .ok_or_else(|| anyhow!("meta.yaml without parent"))?;
        out.push(load(dir)?);
    }
    out.sort_by(|a, b| a.meta.id.cmp(&b.meta.id));
    Ok(out)
}

/// Locate the project root containing `puzzles/`. Tries CWD and walks upward.
pub fn find_project_root() -> Result<PathBuf> {
    let mut cur = std::env::current_dir()?;
    loop {
        if cur.join("puzzles").is_dir() && cur.join("Cargo.toml").is_file() {
            return Ok(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    Err(anyhow!(
        "could not find project root (no Cargo.toml + puzzles/ ancestor of cwd)"
    ))
}
