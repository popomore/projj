# Projj

Manage git repositories with directory conventions — clone once, find instantly.

[![Crates.io](https://img.shields.io/crates/v/projj.svg)](https://crates.io/crates/projj)
[![CI](https://github.com/popomore/projj/actions/workflows/ci.yml/badge.svg)](https://github.com/popomore/projj/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## The Problem

Git repos pile up. You clone them into `~/code`, `~/projects`, `~/work`, or wherever feels right at the moment. Six months later:

```
~/code/projj
~/projects/old-projj
~/misc/projj-backup
~/work/projj-fork
```

Which one is current? Where did you put that internal GitLab repo? You `find / -name .git` and wait.

## The Solution

Projj gives every repo a predictable home based on its URL — just like `GOPATH` did for Go:

```text
$BASE/
├── github.com/
│   └── popomore/
│       └── projj/
└── gitlab.com/
    └── company/
        └── internal-tool/
```

- **One repo, one location** — no duplicates, no guessing
- **Instant lookup** — fuzzy find with fzf, jump with `p projj`
- **Hooks** — auto-configure git user, register with zoxide, run custom scripts on clone
- **Multi-host** — GitHub, GitLab, Gitee, self-hosted — all organized the same way
- **Zero overhead** — no daemon, no cache, no database, just your filesystem

## Install

```bash
# Cargo
cargo install projj

# Homebrew (after first release)
brew install popomore/tap/projj
```

## Quick Start

```bash
projj init                              # one-time setup
projj add popomore/projj               # clone → ~/projj/github.com/popomore/projj
projj add git@gitlab.com:team/app.git   # clone → ~/projj/gitlab.com/team/app
p projj                                 # jump to repo instantly (shell function)
projj run status --all                  # batch operations across all repos
```

### Shell Integration

Add to `~/.zshrc` (or `~/.bashrc`, `~/.config/fish/config.fish`):

```bash
eval "$(projj shell-setup zsh)"    # zsh
eval "$(projj shell-setup bash)"   # bash
projj shell-setup fish | source    # fish
```

This sets up:

- Tab completions for all commands
- `projj run` completes task names from `[tasks]` config and `~/.projj/tasks/`
- `p()` function for quick navigation

```bash
p projj       # jump to projj
p egg         # multiple matches → fzf selection
p             # browse all repos with fzf
```

## Commands

### projj init

Initialize configuration. Creates `~/.projj/config.toml`, installs built-in tasks to `~/.projj/tasks/`, and shows a summary of your setup (base directories, repos found, hooks, tasks).

### projj add \<repo\>

Clone a repo into the conventional directory structure.

```bash
projj add popomore/projj                                  # short form
projj add git@github.com:popomore/projj.git               # SSH
projj add https://github.com/popomore/projj               # HTTPS
projj add ssh://git@git.gitlab.cn:2224/web/cms.git         # SSH with port
projj add ./local/repo                                     # move local repo
```

Runs `post_add` hooks after cloning. Skips hooks if repo already exists.

### projj find [keyword]

Find a repo by keyword (case-insensitive). Outputs the path to stdout.

- **Single match** — prints path directly
- **Multiple matches** — opens fzf for fuzzy selection with colored group tags (base/domain) and git URL
- **No keyword** — lists all repos for selection
- **No fzf** — falls back to numbered list

### projj remove \<keyword\>

Remove a repo. Searches the same way as `find`, then requires typing `owner/repo` to confirm. Runs `pre_remove` / `post_remove` hooks.

### projj run \<task\> [--all] [--match PATTERN] [-- ARGS...]

Run a task in the current directory, or all repos with `--all`.

```bash
projj run "npm install"                             # raw command
projj run update --all                              # named task in all repos
projj run "git status" --all --match "SeeleAI"      # filter repos by regex
projj run status -- --detail                        # pass args to task after --
```

### projj list [--raw]

List all repositories with grouped display, colored by base directory and domain.

```bash
projj list              # pretty: grouped by base/host, colored, with git URL
projj list --raw        # plain paths, one per line (for piping)
```

## Configuration

`~/.projj/config.toml`

```toml
base = ["/Users/x/projj", "/Users/x/work"]
platform = "github.com"

[tasks]
update = "git fetch && git pull origin -p"
clean = "rm -rf node_modules dist target"
status = "git status --short"

[[hooks]]
event = "post_add"
tasks = ["zoxide"]

[[hooks]]
event = "post_add"
matcher = "github\\.com"
tasks = ["zoxide", "git-config-user"]
env = { GIT_USER_NAME = "popomore", GIT_USER_EMAIL = "me@example.com" }

[[hooks]]
event = "post_add"
matcher = "gitlab\\.com"
tasks = ["zoxide", "git-config-user"]
env = { GIT_USER_NAME = "Other Name", GIT_USER_EMAIL = "other@corp.com" }
```

| Field | Description | Default |
|-------|-------------|---------|
| `base` | Root directory (string or array) | `~/projj` |
| `platform` | Default host for short form `owner/repo` | `github.com` |
| `tasks` | Named tasks (see [Tasks](#tasks)) | `{}` |
| `hooks` | Event-driven hooks (see [Hooks](#hooks)) | `[]` |

## Tasks

Tasks are reusable commands that can be run manually via `projj run` or triggered by hooks.

### Defining Tasks

**Inline** — one-liners in `[tasks]` table:

```toml
[tasks]
update = "git fetch && git pull origin -p"
clean = "rm -rf node_modules dist target"
```

**Script files** — executables in `~/.projj/tasks/`:

```bash
cat > ~/.projj/tasks/notify << 'EOF'
#!/bin/bash
echo "Added $PROJJ_REPO_OWNER/$PROJJ_REPO_NAME"
EOF
chmod +x ~/.projj/tasks/notify
```

### Running Tasks

```bash
projj run update --all                    # inline task
projj run notify --all                    # task file
projj run status -- --detail              # pass arguments after --
projj run "git log -5"                    # raw command (not a named task)
```

Resolution order: `[tasks]` table → `~/.projj/tasks/` file → raw shell command.

### Task Context

When tasks are executed via hooks, they receive repo context via environment variables:

```text
PROJJ_EVENT        — event name (e.g. post_add)
PROJJ_REPO_PATH    — full path to repo
PROJJ_REPO_HOST    — e.g. github.com
PROJJ_REPO_OWNER   — e.g. popomore
PROJJ_REPO_NAME    — e.g. projj
PROJJ_REPO_URL     — e.g. git@github.com:popomore/projj.git
```

These are system-provided variables. Hooks can also pass custom variables via the `env` field (see [Hooks](#hooks)).

JSON is also sent via stdin for richer parsing. When run manually via `projj run`, these variables are not set.

### Built-in Tasks

Installed to `~/.projj/tasks/` on `projj init`.

#### zoxide

Registers the repo path with [zoxide](https://github.com/ajeetdsouza/zoxide) so `z` can jump to it. Silently skips if zoxide is not installed.

```toml
[[hooks]]
event = "post_add"
tasks = ["zoxide"]
```

#### git-config-user

Sets `user.name` and `user.email` for the repo. Reads from custom env vars set in the hook's `env` field (not system-provided `PROJJ_*` variables).

```toml
[[hooks]]
event = "post_add"
matcher = "github\\.com"
tasks = ["git-config-user"]
env = { GIT_USER_NAME = "popomore", GIT_USER_EMAIL = "me@example.com" }

[[hooks]]
event = "post_add"
matcher = "gitlab\\.com"
tasks = ["git-config-user"]
env = { GIT_USER_NAME = "Other Name", GIT_USER_EMAIL = "other@corp.com" }
```

| Env var | Description |
|---------|-------------|
| `GIT_USER_NAME` | Value for `git config user.name` |
| `GIT_USER_EMAIL` | Value for `git config user.email` |

Both are optional. Skips if not set.

#### status

Shows disk usage, git status, and ignored files for a repo.

```bash
projj run status                   # current repo
projj run status --all             # all repos (quick summary)
projj run status -- --detail       # include ignored files breakdown
```

Output example:

```text
📦 1.1G total | 🗃️  .git 2.0M | ✓ clean
📦 1.1G total | 🗃️  .git 2.0M | ✓ clean | 🚫 15437 ignored: target(1.1G, 99%)
📦 6.3M total | 🗃️  .git 2.9M | ✎ 22 dirty | 🚫 5 ignored: .claude(1.3M, 21%) skills📂(852K, 13%)
```

Colors by size: green (<100M), yellow (100M–1G), red (>1G). Respects `NO_COLOR`. In the ignored breakdown, a trailing 📂 marks a bucket where only some files inside the top-level directory are ignored (e.g. `skills📂` means "part of `skills/`"), while a bare name like `target` means the whole directory is ignored.

> **Deprecated**: the previous name `repo-status` still works but prints a warning to stderr and delegates to `status`. It will be removed in a future release — please switch to `projj run status`.

#### clean

Removes git-ignored top-level directories (`target/`, `node_modules/`, `dist/`, ...) — the natural follow-up to `repo-status -- --detail`. Only ever touches paths reported by `git ls-files --others --ignored --exclude-standard --directory`, so untracked new files and tracked changes are never at risk.

```bash
projj run clean                                  # dry-run: list all ignored top-level dirs
projj run clean -- target                        # dry-run, filter by name/glob
projj run clean -- --force                       # interactive: y/N/a/q per dir
projj run clean -- --force --yes                 # non-interactive, remove all
projj run clean -- --force target node_modules   # remove only these, no prompt
projj run clean --all                            # dry-run across all repos
projj run clean --all -- --force                 # interactive across all repos
```

Output example:

```text
🧹 would remove (2 items, 1.4G total):
  target        1.1G
  node_modules  340M
tip: projj run clean -- --force   to remove interactively
```

Interactive prompts: `y` yes, `n`/enter skip, `a` yes to all remaining, `q` quit. If stdin is not a tty and `--yes` was not passed, the script refuses to delete and prints the dry-run list instead. If you define `clean` in your `[tasks]` table, it overrides this built-in.

## Hooks

Hooks trigger tasks automatically at repo lifecycle events. They are the glue between events and tasks.

### Events

| Event | When | cwd |
|-------|------|-----|
| `pre_add` | Before clone/move | Target directory |
| `post_add` | After clone/move | Repo directory |
| `pre_remove` | Before deletion | Repo directory |
| `post_remove` | After deletion | Parent directory |

### Configuration

```toml
[[hooks]]
event = "post_add"                                    # required: event name
matcher = "github\\.com"                              # optional: regex on host/owner/repo
tasks = ["zoxide", "git-config-user"]                 # required: tasks to run in order
env = { GIT_USER_NAME = "popomore" }                       # optional: custom env vars for tasks
```

| Field | Required | Description |
|-------|----------|-------------|
| `event` | Yes | Event name |
| `matcher` | No | Regex against `host/owner/repo`. Omit to match all |
| `tasks` | Yes | List of task names or commands, executed in order. Stops on first failure |
| `env` | No | Custom environment variables passed to tasks (user-defined, not `PROJJ_*`) |

Each entry in `tasks` is resolved the same way as `projj run` (task table → task file → raw command).

### Matcher

The `matcher` field is a regex matched against `host/owner/repo`. Omit to match all repos.

| Matcher | Matches |
|---------|---------|
| *(omitted)* or `*` | All repos |
| `github\\.com` | All GitHub repos |
| `github\\.com/SeeleAI` | All repos under SeeleAI org |
| `github\\.com/popomore/projj` | Exact repo |
| `gitlab\\.com\|gitee\\.com` | GitLab or Gitee repos |

Note: `.` in regex matches any character. Use `\\.` to match a literal dot.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NO_COLOR` | Disable all colored output ([no-color.org](https://no-color.org/)) |
| `PROJJ_HOME` | Override home directory for config location |

## External Tools

Optional integrations. Projj works fine without them.

| Tool | Integration | Without it |
|------|------------|------------|
| [fzf](https://github.com/junegunn/fzf) | Fuzzy search in `find` / `remove` | Numbered list |
| [zoxide](https://github.com/ajeetdsouza/zoxide) | `post_add` hook registers paths | No auto-registration |

```bash
# macOS
brew install fzf zoxide
```

## License

[MIT](LICENSE)
