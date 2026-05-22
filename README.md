# bashlings

Learn bash by solving small puzzles in disposable Podman containers. Inspired
by [`rustlings`](https://github.com/rust-lang/rustlings) and OverTheWire's
[Bandit](https://overthewire.org/wargames/bandit/).

Each puzzle runs in an ephemeral container. Your work lives in a bind-mounted
workspace on the host, so files persist across container lifecycles even
though the container itself is destroyed every time you exit. A second
short-lived container then runs `check.sh` against the same workspace and
decides whether you solved it.

> v1 status: working runner, the Module 1 (Streams and Pipes) starter
> puzzles, and the self-test. Linux only. Podman only.

## Prerequisites

- Linux (developed on Linux Mint 21.x / Ubuntu 22.04+)
- [Podman](https://podman.io/) running rootless
- Rust stable toolchain ([rustup](https://rustup.rs/))

### Setting up Podman

```sh
sudo apt update
sudo apt install -y podman

# Sanity check — both should print without errors:
podman --version
podman info | head -20

# Rootless smoke test — must work without sudo:
podman run --rm docker.io/library/alpine echo ok
```

If that last command fails, rootless subuid/subgid likely isn't set. Check
with `grep $USER /etc/subuid /etc/subgid`; if empty:

```sh
sudo usermod --add-subuids 100000-165535 --add-subgids 100000-165535 $USER
podman system migrate
```

Then re-run the smoke test.

## Install and first run

```sh
git clone <this-repo> bashlings
cd bashlings

# Install the runner into ~/.cargo/bin (re-run after code changes).
cargo install --path .

# Build the base container image (one-time).
bashlings init

# Smoke-test all puzzles against their reference solutions.
bashlings verify

# Start solving.
bashlings next
```

`bashlings` walks up from the current directory to find the repo (it needs
to see `Cargo.toml` + `puzzles/`), so run it from inside the repo or a
subdirectory of it.

## Commands

| Command                      | What it does                                                                |
| ---------------------------- | --------------------------------------------------------------------------- |
| `bashlings init`             | Build the base container image (`bashlings-base:latest`).                   |
| `bashlings list`             | Show modules and progress.                                                  |
| `bashlings status`           | Summary of solved/total and the in-progress puzzle.                         |
| `bashlings next`             | Start the next unsolved puzzle.                                             |
| `bashlings start <id>`       | Start a specific puzzle by id (skips linear ordering).                      |
| `bashlings retry`            | Drop back into the in-progress workspace (files preserved, history lost).  |
| `bashlings reset <id>`       | Wipe the workspace, re-copy the template, run setup again.                  |
| `bashlings hint`             | Show the next progressive hint for the in-progress puzzle.                  |
| `bashlings solution <id>`    | Print the reference solution. Free when solved; prompts otherwise.          |
| `bashlings verify [id]`      | Self-test: assert setup → check-fails → solution → check-passes.            |

Add `-v` / `--verbose` to any command to print what the runner is doing
(workspace paths, podman invocations, exit codes).

## How a puzzle runs

1. `bashlings` reads `puzzles/<id>/meta.yaml` and `README.md`, shows the prompt.
2. The template is copied to `~/.bashlings/workspaces/<run-id>/`.
3. A short-lived container runs `setup.sh` against the workspace.
4. An interactive container drops you into bash at `/puzzle`.
5. When you `exit`, that container is destroyed; the workspace persists.
6. A *fresh* check container runs `check.sh` against the same workspace.
7. Exit 0 = solved. Progress is recorded in `~/.bashlings/progress.json`.

`bashlings retry` re-runs steps 4–6 without re-copying the template (so
your files survive). `bashlings reset <id>` wipes the workspace and starts
clean from step 2.

## Storage

- `~/.bashlings/progress.json` — which puzzles you've solved.
- `~/.bashlings/current.json` — the puzzle you're currently working on.
- `~/.bashlings/workspaces/<run-id>/` — your bind-mounted puzzle copies.
  Kept forever in v1; safe to delete when you're done.

Nothing the user writes lives inside the repo. You can experiment with bash
freely without polluting your git tree.

## Container security

Every puzzle and check container is launched with:

```
--rm
--network=none           (unless meta.yaml sets requires.network: true,
                          in which case the runner prompts before launching)
--read-only              (root filesystem is immutable)
--tmpfs /tmp
--tmpfs /home/bashlings  (writable scratch + $HOME)
--cap-drop=ALL
--security-opt=no-new-privileges
--memory=512m
--pids-limit=256
-u 1000:1000             (non-root user defined in the base image)
-v <workspace>:/puzzle   (the only writable bind mount)
-w /puzzle
```

…and a minimal environment (`PATH`, `HOME`, `SHELL`, `TERM` only — no host
env passed through).

**Note on `--cpus`:** the design doc specifies `--cpus=1`, but the `cpu`
cgroup v2 controller isn't delegated to user slices by default on
Ubuntu/Mint, which would break rootless Podman out of the box. It's
intentionally omitted. The remaining flags still contain the threat
model (no network, no capabilities, read-only fs, capped memory, capped
pid count); a runaway CPU loop in a puzzle is annoying but recoverable
with Ctrl-C or `podman kill`. See `src/container.rs` for the rationale.

## Puzzle authoring contract

Each puzzle is a directory under `puzzles/<module>/<id>/`:

```
puzzles/01-streams/03-redirect-stderr/
├── meta.yaml          # id, title, hints, difficulty, requires.network
├── README.md          # prompt shown to the user
├── setup.sh           # runs once before the user gets the shell
├── check.sh           # exit 0 = solved; runs in a fresh container
└── solution.sh        # reference solution; never shown unless asked
```

Inside the container the workspace is mounted at `/puzzle` and that is
also the working directory. **Do not** `cd` anywhere else in
`setup.sh` / `check.sh` / `solution.sh` — operate on the current working
directory so the scripts stay portable.

- `setup.sh` runs once, must be idempotent against a clean workspace, must
  only write inside `/puzzle`, and must exit 0.
- `check.sh` runs after the user exits, in a fresh container. Exit 0 = solved.
  It must check end-state only (file contents, file existence) — no shell-
  history sniffing, no process inspection. It must not modify the workspace.
- `solution.sh` is run by `bashlings verify` as a self-test, and otherwise
  shown to the user on demand. It must transform a freshly-set-up workspace
  into a state that `check.sh` accepts, using only commands available in
  the base image.

After adding a puzzle, run `bashlings verify` (or push and let CI run it
against every puzzle on every commit).

## Repository layout

```
bashlings/
├── Cargo.toml
├── README.md
├── src/                       # the Rust runner
│   ├── main.rs / lib.rs
│   ├── cli.rs
│   ├── commands/              # one module per CLI verb
│   ├── puzzle.rs              # meta.yaml + discovery
│   ├── workspace.rs           # ~/.bashlings paths, run-id, copy-from-template
│   ├── progress.rs            # progress.json + current.json
│   ├── container.rs           # podman invocation builder
│   └── ui.rs                  # tty/color, prompts
├── puzzles/                   # versioned puzzle content
│   └── 01-streams/...
├── containers/
│   └── Containerfile          # base image (Debian stable + GNU userland)
├── tests/                     # cargo test
├── scripts/
│   └── ci-verify-puzzles.sh   # CI hook: build image + run `bashlings verify`
├── .github/workflows/         # CI: build runner, test, verify puzzles
└── docs/
    ├── design_doc.md          # full v1 architectural agreement
    └── bashlings_curriculum.md  # the 10-module learning path
```

## Out of scope for v1

TUI, remote puzzle repositories, Docker support, Windows/macOS, puzzle
signing, leaderboards, internationalization. See `docs/design_doc.md` for
the full architectural agreement.

## Curriculum

The full learning path is in
[`docs/bashlings_curriculum.md`](docs/bashlings_curriculum.md). Module 1
(Streams and Pipes) ships with v1; later modules will be added over time.

## License

MIT.
