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
cargo install --path .
```

## Quick Start

```bash
# Initialize config
projj init

# Clone a repo
projj add popomore/projj
projj add git@github.com:popomore/projj.git

# Find and jump to a repo
projj find projj

# List all repos
projj list
```

### Shell Integration

Add to `~/.zshrc` for quick navigation:

```bash
p() {
  local dir
  dir=$(projj find "$@")
  [ -n "$dir" ] && cd "$dir"
}
```

Then:

```bash
p projj       # jump to projj
p egg         # multiple matches → fzf selection
p             # browse all repos with fzf
```

## Commands

### projj init

Initialize configuration. Creates `~/.projj/config.toml`.

If [zoxide](https://github.com/ajeetdsouza/zoxide) is installed, syncs all existing repos to zoxide.

### projj add \<repo\>

Clone a repo into the conventional directory structure.

```bash
projj add git@github.com:popomore/projj.git    # SSH
projj add https://github.com/popomore/projj    # HTTPS
projj add popomore/projj                       # short form (uses default platform)
projj add ./local/repo                          # move local repo into structure
```

After cloning:
- Runs `preadd` / `postadd` hooks if configured
- Registers the path with zoxide (if available)
- Prints the target path to stdout

### projj find [keyword]

Find a repo by keyword. Outputs the path to stdout.

- Single match → prints path directly
- Multiple matches → opens fzf for selection (falls back to built-in list if fzf is not installed)
- No keyword → lists all repos for selection
- Multiple base directories → results are grouped by base with colored labels

### projj remove \<keyword\>

Remove a repo. Requires typing `owner/repo` to confirm.

### projj run \<script\> [--all]

Run a script in the current directory, or all repos with `--all`.

```bash
projj run "npm install"          # current directory
projj run "npm install" --all    # all repos
projj run postadd                # run a named hook
```

If the script name matches a hook in config, runs that hook's command.

### projj list

List all repo paths, one per line. Pipe-friendly.

```bash
projj list | fzf
projj list | xargs -I{} git -C {} status
```

## Configuration

`~/.projj/config.toml`

```toml
base = ["/Users/x/projj", "/Users/x/work"]
platform = "github.com"

[hooks]
postadd = "npm install"
preadd = "echo cloning..."

[hooks_config.postadd]
"github.com" = { name = "popomore", email = "me@example.com" }
```

| Field | Description | Default |
|-------|-------------|---------|
| `base` | Root directory (string or array) | `~/projj` |
| `platform` | Default host for short form `owner/repo` | `github.com` |
| `hooks` | Hook name → shell command | `{}` |
| `hooks_config` | Extra config passed via `$PROJJ_HOOK_CONFIG` env | `{}` |

## External Tools

Projj integrates with these tools when available. None are required.

| Tool | Integration |
|------|------------|
| [zoxide](https://github.com/ajeetdsouza/zoxide) | `projj add` registers paths; `projj init` syncs all repos |
| [fzf](https://github.com/junegunn/fzf) | Interactive selection in `find` and `remove` |

## License

[MIT](LICENSE)
