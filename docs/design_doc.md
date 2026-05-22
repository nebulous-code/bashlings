# Coding Agent Prompt: bashlings

## Project summary

Build **bashlings**, a Rust CLI tool for learning bash through puzzles. Inspired by `rustlings` and OverTheWire's Bandit. Each puzzle runs in an isolated Podman container; the user's work is preserved across container lifecycles via a bind-mounted workspace directory. The tool is a single static binary targeting Linux (developed on Linux Mint 21.x / Ubuntu 22.04 base).

This is v1. Scope is intentionally narrow: a working runner, a small starter set of puzzles, a CI-style self-test for puzzle validity. No TUI, no plugin system, no remote puzzle repositories.

## High-level architecture

### Components

1. **The runner** — a Rust CLI binary (`bashlings`) installed on the host.
2. **The puzzles** — versioned directories under `puzzles/` in the same repo.
3. **The base image** — a Containerfile in `containers/` that defines the Linux environment puzzles run in. Built on **Debian stable** (not Ubuntu or Alpine) to keep upstream of the Ubuntu/Mint ecosystem and avoid BusyBox flag differences.
4. **The workspace area** — `~/.bashlings/workspaces/<run-id>/`, where per-run copies of puzzle directories live.
5. **Progress state** — `~/.bashlings/progress.json`.

### The puzzle lifecycle

When the user starts a puzzle:

1. Runner reads the puzzle's `meta.yaml` and `README.md`. Displays the prompt.
2. Runner copies the puzzle directory from `puzzles/<id>/` to `~/.bashlings/workspaces/<run-id>/`.
3. Runner launches a Podman container with the workspace bind-mounted at `/puzzle`. Container is built from the base image. The container runs `setup.sh` once, then drops the user into an interactive bash shell at `/puzzle`.
4. User works, then exits.
5. Container is destroyed (`--rm`). Workspace directory persists on the host.
6. Runner launches a *second* short-lived container (same image, same workspace mount) that runs `check.sh` and captures its exit code and stdout.
7. Runner reports result. Updates `progress.json` on success.
8. On failure, user can `bashlings retry` (relaunches a new container against the existing workspace — files preserved, shell history lost) or `bashlings reset` (wipes workspace, re-copies template, re-runs setup).

### Key insight: the bind mount

Each puzzle's workspace directory on the host is the source of truth. The container is an ephemeral environment that provides the tools (bash, grep, awk, etc.) and isolation. The user's work is just file changes to the bind-mounted directory. When the container dies, the directory remains. Both the puzzle container and the check container mount the same workspace directory, so the check container sees exactly what the user produced.

## The puzzle author's contract

Each puzzle is a directory under `puzzles/`. Layout:

```
puzzles/01-streams/03-redirect-stderr/
├── meta.yaml
├── README.md
├── setup.sh
├── check.sh
└── solution.sh
```

### `meta.yaml`
```yaml
id: 01-streams/03-redirect-stderr
title: Silencing errors while keeping output
module: 01-streams
difficulty: easy
estimated_minutes: 5
hints:
  - "Errors and normal output are on different streams (stdout vs stderr)."
  - "You can redirect stderr alone with 2>."
  - "Sending stderr to /dev/null discards it; consider what to do with stdout."
tags: [redirection, stderr, basics]
requires:
  network: false
  writable_paths: [/puzzle]
```

### `README.md`
The puzzle prompt as shown to the user.

### `setup.sh`
- Runs once, inside the puzzle container, before the user gets the shell.
- Working directory: `/puzzle`.
- Must be idempotent against a clean workspace.
- Must only write inside `/puzzle`.
- Must exit 0. Non-zero is an authoring bug.

### `check.sh`
- Runs after the user exits, in a fresh check container against the workspace.
- Working directory: `/puzzle`.
- Exit 0 = solved. Non-zero = not solved.
- Stdout is shown to the user as the result message.
- Must check end state, not method (no shell history sniffing, no process inspection).
- Must not modify the workspace.

### `solution.sh`
- Reference solution. Run via `bashlings verify` for self-testing; never automatically as part of user flow.
- Transforms a freshly-set-up workspace into a state that `check.sh` accepts.
- Uses only commands available in the base image.
- Doubles as documentation for the canonical approach.

## CLI surface

Implement these commands:

```
bashlings list                       # show modules and progress
bashlings next                       # work on the next unsolved puzzle
bashlings start <puzzle-id>          # work on a specific puzzle
bashlings retry                      # relaunch container against existing workspace
bashlings reset <puzzle-id>          # wipe workspace, re-copy template, restart
bashlings hint                       # show next progressive hint for current puzzle
bashlings solution <puzzle-id>       # reveal the reference solution (read-only display)
bashlings verify [puzzle-id]         # self-test: run solution.sh, assert check.sh passes
bashlings status                     # show overall progress summary
```

A `--verbose` / `-v` flag is available on all commands and prints what the runner is doing (workspace paths, podman invocations, exit codes). Default output is quiet.

Ordering: strict linear by default — puzzle N requires N-1 solved. `start <id>` overrides this and is useful for debugging and skipping around. Re-running an already-solved puzzle is allowed; it just restarts it. The solved flag is not cleared by re-running.

## The self-test (`bashlings verify`)

The most important reliability mechanism in the project. For each puzzle, runs:

1. Create a fresh workspace from `puzzles/<id>/`.
2. Run `setup.sh` in a container against it.
3. Run `check.sh` — **assert it exits non-zero** (initial state is unsolved).
4. Run `solution.sh` in a container against the workspace.
5. Run `check.sh` again — **assert it exits zero** (solution actually works).
6. Clean up workspace.

`bashlings verify` (no args) runs all puzzles. `bashlings verify <id>` runs one. This is also the basis for a CI job that validates every puzzle on every commit.

## Container security configuration

Every puzzle container and check container must be launched with these flags by default:

- `--network=none` (unless puzzle's `meta.yaml` declares `requires.network: true`)
- `--read-only` root filesystem
- `--tmpfs /tmp` (for ephemeral scratch space)
- Workspace mounted at `/puzzle` (the only writable bind mount)
- `--cap-drop=ALL`
- `--security-opt=no-new-privileges`
- `--memory=512m`
- `--cpus=1`
- `--pids-limit=256`
- `--rm` (destroyed on exit)
- Run as a non-root user (defined in the base image as `bashlings` with UID 1000)
- Do not pass host environment variables in. Set a minimal env: `PATH`, `HOME=/home/bashlings`, `TERM`, `SHELL=/bin/bash`.

If a puzzle declares `requires.network: true`, prompt the user before launching:
```
⚠ This puzzle requests network access.
  Continue? [y/N]
```

## Storage layout

```
~/.bashlings/
├── progress.json
├── workspaces/
│   └── <run-id>/          # ephemeral per-run puzzle copies
└── current.json           # tracks the in-progress puzzle for retry/hint commands
```

`progress.json` schema:
```json
{
  "version": 1,
  "puzzles": {
    "01-streams/01-redirect": {
      "status": "solved",
      "attempts": 3,
      "hints_shown": 1,
      "first_solved_at": "2026-05-16T14:23:00Z",
      "last_attempted_at": "2026-05-16T14:23:00Z"
    }
  }
}
```

## Repository layout

```
bashlings/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/                       # the Rust runner
│   ├── main.rs
│   └── ...                    # module structure at your discretion
├── puzzles/                   # versioned puzzle content
│   ├── 01-streams/
│   │   ├── 01-redirect-to-file/
│   │   ├── 02-append-vs-overwrite/
│   │   └── 03-redirect-stderr/
│   └── ...
├── containers/
│   └── Containerfile          # the base image (Debian stable)
├── tests/                     # Rust integration tests for the runner
├── scripts/
│   └── ci-verify-puzzles.sh   # invoked by CI to run `bashlings verify`
└── .github/workflows/         # CI: build runner, run unit tests, run verify
```

Puzzles live in-repo for v1. Pulling puzzles from external sources (separate repos, registries) is explicitly out of scope but the design should not preclude it later.

## Implementation notes

- **Language:** Rust, stable channel. Single binary.
- **CLI parsing:** `clap` (derive macros are fine).
- **Config parsing:** `serde` + `serde_yaml` + `serde_json`.
- **Process orchestration:** `std::process::Command`, shelling out to `podman`. Do not use a Rust Podman client crate for v1 — shelling out is easier to debug (commands are copy-pasteable) and keeps dependencies thin. Capture stdout/stderr/exit code from each `podman run` invocation.
- **Async:** Not needed. Stay synchronous for v1.
- **Error handling:** Your choice (`anyhow` for the binary is reasonable).
- **Output:** Plain CLI, no TUI. Use color when stdout is a tty (`is-terminal` or similar), plain text otherwise. Print what the runner is doing in verbose mode (`-v`); stay quiet by default.
- **Logging:** `tracing` or `env_logger`, your call.

## What to deliver

1. A working `bashlings` binary that implements the CLI surface above.
2. The base Containerfile in `containers/`, built on **Debian stable**, with an image containing bash, GNU coreutils, findutils, grep, sed, gawk, ripgrep, jq, curl, wget, tar, gzip, less, vim, nano. Image tagged as `bashlings-base:latest` on local build.
3. A starter set of 3–5 puzzles for Module 1 (Streams and Pipes) following the contract above. Use them to exercise the runner end-to-end.
4. `bashlings verify` working against the starter puzzles, all passing.
5. A `README.md` with: installation, first-run instructions (build base image, run `bashlings next`), and the puzzle authoring contract.
6. CI workflow that builds the runner, runs `cargo test`, builds the base image, and runs `bashlings verify`.

## Out of scope for v1

- TUI / full-screen interface
- Remote puzzle repositories or plugin system
- Puzzle signing / cryptographic verification
- Windows or macOS support
- Docker support (Podman only)
- "Strict mode" with extra paranoia for untrusted puzzles
- Shell other than bash
- Internationalization
- Puzzle search / tagging UI beyond `bashlings list`
- Stats, leaderboards, time tracking beyond basic timestamps

## Design conversations expected

The user (project owner) expects there will be a follow-up design conversation with you about implementation details. Treat this prompt as the architectural agreement and ask before deviating from it. For tactical choices not specified here (exact crate choices, module layout, error types, output formatting details), use your judgment and move forward.

## Open questions to confirm before starting

1. Should `bashlings` install the base image automatically on first run, or require an explicit `bashlings init`? (Recommend: explicit, so the user knows when Podman is being invoked.)
2. Should `solution.sh` be displayed via `bashlings solution <id>` even if the puzzle is unsolved, or gated behind confirmation? (Recommend: confirmation prompt — "This will reveal the answer. Continue? [y/N]".)
3. Should workspace directories be auto-cleaned after successful solves, or kept for inspection? (Recommend: keep last N per puzzle, configurable.)
