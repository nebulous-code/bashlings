use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const IMAGE_TAG: &str = "bashlings-base:latest";

/// Builder for a `podman run` invocation. Holds the security defaults from the design doc.
#[derive(Debug, Clone)]
pub struct PodmanRun {
    pub workspace: PathBuf,
    pub image: String,
    pub allow_network: bool,
    pub interactive: bool,
    pub command: Vec<String>,
}

impl PodmanRun {
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
            image: IMAGE_TAG.to_string(),
            allow_network: false,
            interactive: false,
            command: Vec::new(),
        }
    }

    pub fn allow_network(mut self, yes: bool) -> Self {
        self.allow_network = yes;
        self
    }
    pub fn interactive(mut self, yes: bool) -> Self {
        self.interactive = yes;
        self
    }
    pub fn command<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command = args.into_iter().map(Into::into).collect();
        self
    }

    /// Build the argument vector that will be passed to `podman` (without "podman" itself).
    pub fn build_args(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        v.push("run".into());
        v.push("--rm".into());
        if self.interactive {
            v.push("-i".into());
            v.push("-t".into());
        }
        if !self.allow_network {
            v.push("--network=none".into());
        }
        v.push("--read-only".into());
        v.push("--tmpfs".into());
        v.push("/tmp".into());
        v.push("--tmpfs".into());
        v.push("/home/bashlings".into());
        v.push("--cap-drop=ALL".into());
        v.push("--security-opt=no-new-privileges".into());
        v.push("--memory=512m".into());
        // The design doc specifies `--cpus=1`, but the `cpu` cgroup v2 controller
        // is not delegated to user slices by default on Ubuntu/Mint, so requiring
        // it breaks rootless `podman run` out of the box. Dropping the throttle
        // costs us a defense against a *runaway* puzzle pegging host CPU; it does
        // not weaken the protections that matter (network, capabilities, fs,
        // memory, pids). If we ever support untrusted third-party puzzles, this
        // is where a "strict mode" should reintroduce a CPU cap.
        v.push("--pids-limit=256".into());
        v.push("-u".into());
        v.push("1000:1000".into());
        v.push("-w".into());
        v.push("/puzzle".into());
        v.push("-v".into());
        // `:U` chowns the bind-mount to the container's runtime UID — required
        // for rootless Podman because UID 1000 inside the container maps via
        // subuid to a different UID on the host, which doesn't own the
        // workspace dir. `:Z` relabels for SELinux; it's a no-op on Ubuntu/Mint
        // but harmless to include.
        v.push(format!("{}:/puzzle:Z,U", self.workspace.display()));
        // Minimal env.
        for kv in [
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            "HOME=/home/bashlings",
            "SHELL=/bin/bash",
        ] {
            v.push("-e".into());
            v.push(kv.into());
        }
        // TERM passed through if set.
        if let Ok(term) = std::env::var("TERM") {
            v.push("-e".into());
            v.push(format!("TERM={term}"));
        }
        v.push(self.image.clone());
        v.extend(self.command.iter().cloned());
        v
    }

    /// Spawn and wait. stdio inherited for interactive runs; captured otherwise.
    pub fn spawn(&self) -> Result<RunOutput> {
        let args = self.build_args();
        tracing::debug!(podman_args = ?args, "podman run");
        let mut cmd = Command::new("podman");
        cmd.args(&args);
        if self.interactive {
            cmd.stdin(Stdio::inherit());
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
            let status = cmd.status().context("spawning podman")?;
            Ok(RunOutput {
                exit: status.code().unwrap_or(-1),
                stdout: String::new(),
                stderr: String::new(),
            })
        } else {
            let out = cmd.output().context("spawning podman")?;
            Ok(RunOutput {
                exit: out.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunOutput {
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run `setup.sh` inside the workspace.
pub fn run_setup(workspace: &Path, allow_network: bool) -> Result<RunOutput> {
    PodmanRun::new(workspace)
        .allow_network(allow_network)
        .command(["bash", "/puzzle/setup.sh"])
        .spawn()
}

/// Run `check.sh` inside the workspace.
pub fn run_check(workspace: &Path, allow_network: bool) -> Result<RunOutput> {
    PodmanRun::new(workspace)
        .allow_network(allow_network)
        .command(["bash", "/puzzle/check.sh"])
        .spawn()
}

/// Run `solution.sh` inside the workspace.
pub fn run_solution(workspace: &Path, allow_network: bool) -> Result<RunOutput> {
    PodmanRun::new(workspace)
        .allow_network(allow_network)
        .command(["bash", "/puzzle/solution.sh"])
        .spawn()
}

/// Launch an interactive bash shell in the workspace.
pub fn run_interactive_shell(workspace: &Path, allow_network: bool) -> Result<RunOutput> {
    PodmanRun::new(workspace)
        .allow_network(allow_network)
        .interactive(true)
        .command(["bash"])
        .spawn()
}

/// Build the base image from the given containers directory.
pub fn build_image(containers_dir: &Path, verbose: bool) -> Result<()> {
    let mut cmd = Command::new("podman");
    cmd.arg("build")
        .arg("-t")
        .arg(IMAGE_TAG)
        .arg(containers_dir);
    tracing::debug!(?cmd, "podman build");
    if verbose {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }
    let status = cmd.status().context("spawning podman build")?;
    if !status.success() {
        bail!("podman build failed (exit {})", status.code().unwrap_or(-1));
    }
    Ok(())
}
