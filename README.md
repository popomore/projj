# Projj

Manage git repositories with directory conventions.

## Why?

Every git repo gets a predictable home based on its URL:

```
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
# Initialize config + install built-in hooks
projj init

# Clone a repo
projj add popomore/projj

# Find and jump to a repo
projj find projj

# List all repos
projj list
```

### Shell Integration

Add to `~/.zshrc` (or `~/.bashrc`, `~/.config/fish/config.fish`):

```bash
eval "$(projj shell-setup zsh)"    # zsh
eval "$(projj shell-setup bash)"   # bash
projj shell-setup fish | source    # fish
```

Then:

```bash
p projj       # jump to projj
p egg         # multiple matches → fzf selection
p             # browse all repos with fzf
```

## Commands

### projj init

Initialize configuration. Creates `~/.projj/config.toml` and installs built-in hooks to `~/.projj/hooks/`.

Also runs `post_add` hooks for all existing repos (e.g. syncs to zoxide).

### projj add \<repo\>

Clone a repo into the conventional directory structure.

```bash
projj add git@github.com:popomore/projj.git              # SSH
projj add https://github.com/popomore/projj              # HTTPS
projj add ssh://git@git.gitlab.cn:2224/web/cms.git        # SSH with port
projj add popomore/projj                                  # short form (uses default platform)
projj add ./local/repo                                     # move local repo into structure
```

After cloning, runs `post_add` hooks (e.g. zoxide registration, git user config).

### projj find [keyword]

Find a repo by keyword (case-insensitive). Outputs the path to stdout.

- Single match → prints path with group info
- Multiple matches → opens fzf for selection (falls back to built-in list without fzf)
- No keyword → lists all repos for selection
- Results show colored group tags (base/domain) and git URL

### projj remove \<keyword\>

Remove a repo. Requires typing `owner/repo` to confirm. Runs `pre_remove` / `post_remove` hooks.

### projj run \<script\> [--all] [--match PATTERN]

Run a script in the current directory, or all repos with `--all`.

```bash
projj run "npm install"                          # current directory
projj run "npm install" --all                    # all repos
projj run update --all                           # run named script
projj run "git status" --all --match "SeeleAI"   # filter repos by regex
```

Script name is resolved in order:
1. `[scripts]` table in config
2. Executable in `~/.projj/hooks/`
3. Raw shell command

### projj list

List all repo paths, one per line. Pipe-friendly.

```bash
projj list | fzf
projj list | wc -l
projj list | xargs -I{} git -C {} pull
```

## Configuration

`~/.projj/config.toml`

```toml
base = ["/Users/x/projj", "/Users/x/work"]
platform = "github.com"

[scripts]
clean = "rm -rf node_modules && rm -rf dist"
update = "git fetch && git pull origin -p"
status = "git status --short"

[[hooks]]
event = "post_add"
command = "zoxide"                    # → ~/.projj/hooks/zoxide

[[hooks]]
event = "post_add"
matcher = "github\\.com"
command = "git_config_user"           # → ~/.projj/hooks/git_config_user
env = { PROJJ_GIT_NAME = "popomore", PROJJ_GIT_EMAIL = "me@example.com" }

[[hooks]]
event = "post_add"
matcher = "gitlab\\.com"
command = "git_config_user"
env = { PROJJ_GIT_NAME = "Other Name", PROJJ_GIT_EMAIL = "other@example.com" }
```

### Config Fields

| Field | Description | Default |
|-------|-------------|---------|
| `base` | Root directory (string or array) | `~/projj` |
| `platform` | Default host for short form `owner/repo` | `github.com` |
| `scripts` | Named scripts, reusable by hooks and `projj run` | `{}` |
| `hooks` | Event-driven hook entries (see below) | `[]` |

### Hook System

Hooks fire at repo lifecycle events:

| Event | When | cwd |
|-------|------|-----|
| `pre_add` | Before clone/move | Target directory (may not exist yet) |
| `post_add` | After clone/move | Repo directory |
| `pre_remove` | Before deletion | Repo directory |
| `post_remove` | After deletion | Parent directory |

Each hook entry:

| Field | Required | Description |
|-------|----------|-------------|
| `event` | Yes | Event name |
| `matcher` | No | Regex against `host/owner/repo`. Omit to match all |
| `command` | Yes | Script name or shell command |
| `env` | No | Extra environment variables |

Hooks receive context via environment variables:

```
PROJJ_EVENT        — event name
PROJJ_REPO_PATH    — full path to repo
PROJJ_REPO_HOST    — e.g. github.com
PROJJ_REPO_OWNER   — e.g. popomore
PROJJ_REPO_NAME    — e.g. projj
PROJJ_REPO_URL     — e.g. git@github.com:popomore/projj.git
```

And JSON via stdin for richer parsing.

### Built-in Hooks

Installed to `~/.projj/hooks/` on `projj init`:

| Hook | Description |
|------|-------------|
| `zoxide` | Registers repo path with zoxide (if installed) |
| `git_config_user` | Sets `user.name` / `user.email` from `PROJJ_GIT_NAME` / `PROJJ_GIT_EMAIL` env |

### Custom Hooks

Drop executable scripts into `~/.projj/hooks/`:

```bash
cat > ~/.projj/hooks/notify << 'EOF'
#!/bin/bash
echo "Added $PROJJ_REPO_OWNER/$PROJJ_REPO_NAME"
EOF
chmod +x ~/.projj/hooks/notify
```

Then reference by name in config:

```toml
[[hooks]]
event = "post_add"
command = "notify"
```

## External Tools

Projj integrates with these tools when available. **None are required** — projj works fine without them, just with a simpler experience.

| Tool | Integration | Without it |
|------|------------|------------|
| [fzf](https://github.com/junegunn/fzf) | Fuzzy search, colored groups in `find` / `remove` | Falls back to numbered list |
| [zoxide](https://github.com/ajeetdsouza/zoxide) | `post_add` hook registers paths for `z` navigation | No auto-registration |

Install (optional):

```bash
# macOS
brew install fzf zoxide

# Ubuntu/Debian
sudo apt install fzf
curl -sSfL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | sh
```

## License

[MIT](LICENSE)
