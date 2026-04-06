# Projj

Manage git repositories with directory conventions.

## Why?

Every git repo gets a predictable home based on its URL:

```text
$BASE/
├── github.com/
│   └── popomore/
│       └── projj/
└── gitlab.com/
    └── popomore/
        └── projj/
```

No more `~/code/misc/old-projj-backup`. Clone once, find instantly.

## Install

```bash
# Cargo
cargo install projj

# Homebrew (after first release)
brew install popomore/tap/projj
```

## Quick Start

```bash
projj init                              # initialize config + install built-in tasks
projj add popomore/projj               # clone a repo
projj find projj                        # find and jump to a repo
projj list                              # list all repos with grouped display
projj run repo-status --all             # check disk usage and git status across repos
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
projj run repo-status -- --detail                   # pass args to task after --
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
projj run repo-status -- --detail         # pass arguments after --
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

#### repo-status

Shows disk usage, git status, and ignored files for a repo.

```bash
projj run repo-status                   # current repo
projj run repo-status --all             # all repos (quick summary)
projj run repo-status -- --detail       # include ignored files breakdown
```

Output example:

```text
📦 1.1G total | 🗃️  .git 2.0M | ✓ clean
📦 1.1G total | 🗃️  .git 2.0M | ✓ clean | 🚫 15437 ignored: target(1.1G, 99%)
```

Colors by size: green (<100M), yellow (100M–1G), red (>1G). Respects `NO_COLOR`.

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
