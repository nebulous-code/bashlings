# A Bash Curriculum: From Comfortable to Fluent

A learning path for someone who can navigate (`cd`, `ls`) and use specific tools daily (ssh, scp, git, nvim) but wants to close the gaps around **unknown unknowns** and **command composition (piping)**.

Scope: comfortable in the CLI, not Linux internals. No systemd, no kernel debugging, no filesystem layout.

---

## The Core Idea First

On Unix, almost everything is text, and almost every tool reads text from stdin and writes text to stdout. **A command is a function from text to text.** Pipes chain functions. Redirection connects them to files.

Once that clicks, the question stops being *"what command does X"* and becomes *"what sequence of small transformations gets me from input to output."* That mental shift is the actual goal of this curriculum.

---

## Shell Mechanics (the grammar)

These are not commands — they are how the shell works. Without them, commands feel like magic incantations.

### Streams and redirection
Every process has three streams: **stdin (0)**, **stdout (1)**, **stderr (2)**.
- `>` redirect stdout to file (overwrite)
- `>>` redirect stdout to file (append)
- `<` read stdin from file
- `2>` redirect stderr
- `2>&1` merge stderr into stdout
- `&>` redirect both at once

Order matters: `command > file 2>&1` is different from `command 2>&1 > file`. Understanding why is a small enlightenment.

### Pipes
`a | b` connects a's stdout to b's stdin. Each command in the pipeline runs **concurrently**, streaming as it goes. `cat huge.log | grep error` doesn't wait for `cat` to finish.

### Command substitution
`$(command)` runs a command and substitutes its output.
```bash
echo "Today is $(date)"
files=$(ls *.txt)
```
The older backtick form `` `command` `` works too, but `$(...)` nests cleanly.

### Globbing
`*`, `?`, `[abc]`, `{a,b,c}`. **The shell expands globs before passing arguments to the command.** `rm *.txt` doesn't pass `*.txt` to `rm` — the shell expands it first. This explains a lot of "weird" behavior.

### Quoting
- Single quotes: preserve everything literally.
- Double quotes: allow variable and command substitution.
- No quotes: shell splits on whitespace and globs.

This is the source of approximately 40% of bash bugs.

### Exit codes and chaining
Every command returns 0 (success) or non-zero (failure).
- `a && b` — run `b` only if `a` succeeded
- `a || b` — run `b` only if `a` failed
- `a ; b` — run both regardless

### Variables and environment
- `FOO=bar` (no spaces around `=`!) — shell variable
- `export FOO=bar` — visible to child processes
- `$FOO` or `${FOO}` — read it
- `env`, `printenv`, `set` — show what's defined

### History and line editing
- `Ctrl-R` — search history
- `!!` — rerun last command
- `!$` — last argument of previous command
- `Ctrl-A` / `Ctrl-E` — start/end of line
- `Ctrl-W` — delete a word back
- `Ctrl-U` — delete to start of line

---

## The Commands, Grouped by Purpose

### Finding things
- **`find`** — recursively walk a tree with filters. Powerful, awkward syntax. Learn deeply.
- **`locate` / `mlocate`** — fast filename search against a prebuilt database.
- **`which` / `type` / `command -v`** — where does this command live? `type` is most informative.

### Searching inside files
- **`grep`** — the workhorse. Flags: `-i` (case-insensitive), `-r` (recursive), `-n` (line numbers), `-v` (invert), `-E` (extended regex), `-l` (just filenames), `-C 3` (3 lines context).
- **`ripgrep` (`rg`)** — modern, faster, respects `.gitignore`. Worth installing.

### Slicing and dicing text
The heart of piping fluency. Each does one transformation.
- **`cat`** — concatenate / stream a file. Often overused.
- **`head` / `tail`** — first/last N lines. `tail -f` follows a file as it grows.
- **`wc`** — word, line, character count. `wc -l` is the common case.
- **`sort`** — sort lines. `-n` numeric, `-r` reverse, `-u` unique, `-k` by column.
- **`uniq`** — collapse adjacent duplicates. Classic combo: `sort | uniq -c | sort -rn`.
- **`cut`** — extract columns. `cut -d',' -f2,4` gets fields 2 and 4 of CSV.
- **`tr`** — translate/delete characters. `tr 'A-Z' 'a-z'` lowercases.
- **`tee`** — split a stream to file AND stdout. `cmd | tee log.txt | grep error`.
- **`paste`** — join lines from multiple files side by side.
- **`column`** — pretty-print tabular data.

### Heavyweights of text processing
- **`sed`** — stream editor. Mostly `sed 's/old/new/g' file`, but capable of much more.
- **`awk`** — a small programming language for tabular text. `awk '{print $2}'` prints the 2nd field. Worth a dedicated afternoon.
- **`jq`** — `awk` for JSON. If you touch APIs, this is mandatory.

### Files and directories
- **`cp`, `mv`, `rm`, `mkdir`, `rmdir`, `touch`, `ln`** — basics with flags. `cp -r`, `rm -i`, `ln -s`.
- **`stat`** — detailed file metadata (timestamps, permissions, inode).
- **`du`** — disk usage. `du -sh *` is the "what's taking space here" idiom.
- **`df`** — disk free, by filesystem.
- **`file`** — guess what a file actually is, ignoring extension.
- **`tree`** — visual directory tree. `apt install tree`.

### Permissions and ownership
- **`chmod`** — change file mode. Both numeric (`chmod 755`) and symbolic (`chmod u+x,g-w`). The model — owner/group/other × read/write/execute — matters more than the flags.
- **`chown`** — change owner/group. `chown user:group file`.
- **`umask`** — default-permissions mask applied to new files.

### Archives and compression
- **`tar`** — bundle directories. `tar -czf archive.tar.gz dir/` to create, `tar -xzf archive.tar.gz` to extract. Mnemonic: "**c**reate **z**ipped **f**ile" / "e**x**tract **z**ipped **f**ile".
- **`gzip` / `gunzip`, `zip` / `unzip`, `xz`** — individual formats.

### Processes
- **`ps`** — snapshot of processes. `ps aux` is the common incantation.
- **`top` / `htop`** — live process viewer. `htop` is nicer; install it.
- **`kill`** — send signals. `kill -9` is nuclear; `kill -15` (SIGTERM, default) is polite.
- **`pgrep` / `pkill`** — find/kill by name instead of PID.
- **`jobs`, `fg`, `bg`, `&`, `Ctrl-Z`** — managing backgrounded shell jobs.
- **`nohup`, `disown`** — keep processes alive after logout.

### Networking
- **`curl`** — HTTP client. Swiss Army knife of "talk to a URL."
- **`wget`** — simpler downloader.
- **`dig` / `nslookup`** — DNS lookups.
- **`ping`** — is that host reachable.
- **`netstat` / `ss`** — what ports am I listening on / connected to. `ss -tlnp` is the modern incantation.
- **`rsync`** — smarter `scp`. Incremental, resumable, mirrors directories.

### Time and scheduling
- **`date`** — current date, with format strings. `date +%Y-%m-%d` for ISO dates in scripts.
- **`sleep`** — pause. Useful in loops and scripts.
- **`time`** — how long does this command take. `time slow_thing`.
- **`watch`** — re-run a command every N seconds. `watch -n 1 df -h`.
- **`at`** — schedule a one-shot command for later.
- **`cron` / `crontab -e`** — recurring scheduled jobs. Worth knowing the five-field syntax exists.

### Nice to have
- **`xargs`** — turn stdin into command-line arguments. `find . -name "*.log" | xargs rm`. Critical for composing tools that don't read stdin natively. Use `-print0` / `-0` to handle filenames with spaces.
- **`man`, `info`, `tldr`, `--help`** — the help system. `tldr` gives example-driven summaries that are often more useful than full man pages.
- **`history`** — your command history, searchable.
- **`alias`** — make shortcuts. `alias ll='ls -lah'` in `.bashrc`.
- **`env`, `which`, `echo`** — introspecting what the shell thinks is going on.

### Out of scope (for now)
- `systemctl` / systemd
- `mount`, `fdisk`, `lsblk`
- `iptables` / `nftables`
- `strace`, `lsof`

---

## Curriculum: 10-Module Path

Order is important. Each module builds on the last. Most modules should have ~5–10 puzzles.

### Module 1: Streams and Pipes
**Goal:** Force composition into muscle memory before learning the tools.
**Commands:** `cat`, `echo`, `wc`, `head`, `tail`, plus `|`, `>`, `>>`, `<`, `2>`, `2>&1`.
**Concepts:** stdin/stdout/stderr, redirection, piping, "everything is a stream."
**Sample puzzles:**
- Save the output of `ls` to a file.
- Count how many lines are in `data.txt`.
- Show the last 20 lines of one file followed by the first 5 of another, redirected to a third.
- A command produces both useful output and noisy errors — silence one but keep the other.

### Module 2: Finding and Searching
**Goal:** Locate files by name/attribute and content.
**Commands:** `find`, `grep` (and `rg` if installed), `locate`, `which`, `type`.
**Concepts:** recursive search, regex basics (literal, `.`, `*`, `^`, `$`, `[...]`).
**Sample puzzles:**
- Find all files larger than 1MB modified in the last week.
- Find every file containing the string "TODO" and list just the filenames.
- Find files whose names match a pattern AND that contain a pattern inside.

### Module 3: Text Mangling
**Goal:** Build the `sort | uniq -c | sort -rn` reflex.
**Commands:** `cut`, `sort`, `uniq`, `tr`, `wc`, `tee`, `paste`, `column`.
**Concepts:** delimiters, fields, pipelines as data flow.
**Sample puzzles:**
- Given a log file of IPs, find the 10 most frequent.
- Convert a CSV to uppercase, then extract column 3.
- From a list of names, find duplicates.

### Module 4: sed and awk
**Goal:** Substitution and field-aware processing. Treat as two separate sub-modules.
**Commands:** `sed`, `awk`.
**Concepts:** regex (more depth), `awk` fields (`$1`, `$NF`, `NR`), basic `awk` conditions.
**Sample puzzles:**
- Replace every occurrence of "foo" with "bar" in a file, in place.
- From a CSV, print only rows where column 3 is greater than 100.
- Sum a column of numbers.
- Reformat dates from `YYYY-MM-DD` to `DD/MM/YYYY`.

### Module 5: `find` with Actions
**Goal:** Realize `find` is itself a small programming environment.
**Commands:** `find -exec`, `find -delete`, `xargs`, `xargs -0` / `find -print0`.
**Concepts:** combining search with action, handling filenames safely (spaces, newlines), dry-runs.
**Sample puzzles:**
- Delete all `.tmp` files older than 30 days.
- For every `.jpg` in a tree, print its size.
- Find files matching a pattern and `chmod` them all at once.

### Module 6: Permissions
**Goal:** Internalize the owner/group/other × r/w/x model.
**Commands:** `chmod`, `chown`, `umask`, `stat`, `ls -l`.
**Concepts:** numeric vs symbolic modes, the meaning of `x` on directories, why `umask` matters.
**Sample puzzles:**
- Make a script executable only by you.
- Recursively make every `.sh` in a tree executable, but nothing else.
- Read the permissions output of `ls -l` and translate it to a numeric mode.

### Module 7: Archives
**Goal:** Stop Googling tar flags.
**Commands:** `tar`, `gzip`/`gunzip`, `zip`/`unzip`, `xz`.
**Concepts:** the c/x/z/f mnemonic, archive vs compression as separate concerns.
**Sample puzzles:**
- Bundle a directory into a `.tar.gz`.
- Unpack a `.tar.bz2` into a specific directory.
- List the contents of an archive without extracting.

### Module 8: Processes
**Goal:** See, signal, and background processes confidently.
**Commands:** `ps`, `top`/`htop`, `kill`, `pgrep`, `pkill`, `jobs`, `fg`, `bg`, `&`, `nohup`, `disown`.
**Concepts:** PIDs, signals (SIGTERM vs SIGKILL), foreground vs background jobs.
**Sample puzzles:**
- Start a long-running command in the background, do other work, bring it back.
- Find every process owned by a specific user and signal them all.
- Run something that survives your shell closing.

### Module 9: Networking
**Goal:** Reach across the wire from the terminal.
**Commands:** `curl`, `wget`, `ping`, `dig`/`nslookup`, `ss` (or `netstat`), `rsync`.
**Concepts:** HTTP verbs at the CLI, DNS resolution, what "listening on a port" means, rsync's incremental model.
**Sample puzzles:**
- `curl` an API and extract a field with `jq`.
- Find which process is listening on port 8080.
- Mirror a remote directory locally with `rsync`, dry-run first.
- Resolve a domain to its IP and reverse-resolve it.

### Module 10: Scheduling and Time
**Goal:** Repeat work without sitting there.
**Commands:** `date`, `sleep`, `time`, `watch`, `at`, `cron` / `crontab`.
**Concepts:** cron's five-field syntax, `at` for one-shots, `watch` for polling, formatting timestamps.
**Sample puzzles:**
- Schedule a script to run every Sunday at 3am.
- Watch a directory's size every 2 seconds.
- Time how long a command takes; compare two implementations.
- Build a timestamped log filename inside a script.

### Module 11: Nice to Have
**Goal:** Round out the toolkit and the help system.
**Commands:** `xargs`, `man`, `tldr`, `--help`, `history`, `alias`, `env`, `echo`.
**Concepts:** reading man pages, the SYNOPSIS line as grammar, building aliases that stick.
**Sample puzzles:**
- Build a useful alias and put it in your `.bashrc`.
- Use `xargs` to feed `find` results into a command that doesn't read stdin.
- Read a man page you've never seen and explain a flag.

### Module 12: Composition Challenges
**Goal:** Fluency. Open-ended puzzles needing 4–6 tools chained.
**Sample puzzles:**
- From a directory of logs, find the user who triggered the most errors yesterday.
- Build a one-liner that lists the 10 largest files anywhere on disk, ignoring permission errors.
- Take a JSON API response, extract a field, sort by frequency, format as a table.
- From a git log, count commits per author per month.

---

## Cross-Cutting Concepts to Internalize

### The pipeline-first mindset
Decompose every task. "Unique IPs sorted by frequency from a log" becomes: extract IPs → count → sort by count. Each step is a small, comprehensible transformation. **Force yourself not to write it in Python**, even when bash is harder — the cognitive habit of composition is what you're training.

### Regular expressions
Not bash-specific but inescapable. `grep`, `sed`, `awk` all consume regex. Learn: character classes, quantifiers (`*`, `+`, `?`, `{n,m}`), anchors (`^`, `$`), groups (`(...)`), alternation (`|`).

### Exit codes as control flow
Treat every command as returning a boolean. `if grep -q pattern file; then ...; fi` works because `if` just inspects the exit code. This unlocks scripting.

### Idempotency and dry-runs
Many destructive tools have `--dry-run` or equivalent. Reach for it. **Workflow: build the pipeline → dry-run it → execute it.**

### Shell config files
- `.bashrc` — runs for interactive shells
- `.bash_profile` — runs for login shells
- `.profile` — POSIX equivalent

### Reading man pages
Structure: NAME, SYNOPSIS, DESCRIPTION, OPTIONS, sometimes EXAMPLES. The SYNOPSIS line tells you the grammar. They become readable with practice.

---

## How to Shrink the Unknown-Unknowns Gap

**Read other people's one-liners.** Build a habit: when you see a clever shell snippet in a blog post, Stack Overflow answer, Makefile, or colleague's script — paste it into [explainshell.com](https://explainshell.com) and read what each piece does. Over months this builds enormous passive vocabulary. You start recognizing patterns ("oh, that's a `find ... -print0 | xargs -0` pattern for filenames with spaces") and reaching for them.

**Force-bash daily tasks.** When you'd reach for a GUI or copy-paste, do it in bash instead — even slowly. The friction is the learning.
