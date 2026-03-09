# kort

Fast and safe abbreviation expansion for zsh.

kort pre-validates your abbreviations at compile time — catching conflicts with existing commands before they cause problems — then uses a binary cache for instant expansion at runtime.

## Installation

### Build from source

```bash
cargo install --path .
```

### Setup

1. Generate a config file:

```bash
kort init
```

This creates `~/.config/kort/kort.toml`.

2. Source the zsh integration in your `.zshrc`:

```bash
source /path/to/kort/shells/zsh/kort.zsh
```

3. Compile your config:

```bash
kort compile
```

## Configuration

Edit `~/.config/kort/kort.toml` to define your abbreviations.

### Regular Abbreviations

Expand only at command position (the beginning of a command):

```toml
[[abbr]]
keyword = "g"
expansion = "git"

[[abbr]]
keyword = "gc"
expansion = "git commit"
```

Typing `g` then pressing Space expands to `git `. But `echo g` does not expand, because `g` is not in command position.

### Global Abbreviations

Expand anywhere in the line:

```toml
[[abbr]]
keyword = "NE"
expansion = "2>/dev/null"
global = true
```

`curl example.com NE` expands to `curl example.com 2>/dev/null`.

### Contextual Abbreviations

Expand only when surrounding text matches regex patterns:

```toml
[[abbr]]
keyword = "main"
expansion = "main --branch"
context.lbuffer = "^git (checkout|switch) "
```

`main` expands to `main --branch` only after `git checkout` or `git switch`.

### Placeholders

Use `{{name}}` to mark positions where you want to type after expansion:

```toml
[[abbr]]
keyword = "gc"
expansion = "git commit -m '{{message}}'"
```

`gc` expands to `git commit -m ''` with the cursor placed between the quotes. Press Tab to jump to the next placeholder if there are multiple.

### Evaluate Mode

Execute a shell command and insert its output:

```toml
[[abbr]]
keyword = "TODAY"
expansion = "date +%Y-%m-%d"
evaluate = true
global = true
```

`TODAY` expands to the current date, e.g. `2026-03-08`.

### Settings

```toml
[settings]
strict = false  # treat suffix conflicts as errors (default: false)
```

## Conflict Detection

When you run `kort compile`, kort scans your `$PATH` and checks zsh builtins to detect abbreviations that shadow existing commands.

| Conflict Type | Default Behavior |
|---|---|
| Exact match with a command in `$PATH` | Error |
| Suffix match (e.g. `cc` matches `gcc`) | Warning (Error with `--strict`) |
| zsh builtin (e.g. `cd`, `echo`) | Error |

To allow a specific conflict:

```toml
[[abbr]]
keyword = "gs"
expansion = "git status --short"
allow_conflict = true
```

## Key Bindings

The zsh integration sets up three key bindings:

| Key | Action |
|---|---|
| Space | Expand abbreviation, then insert space |
| Enter | Expand abbreviation, then execute |
| Tab | Jump to next `{{placeholder}}` (falls back to normal completion) |

## Adding Abbreviations from the CLI

Instead of editing `kort.toml` by hand, you can use `kort add`:

### Non-interactive

```bash
kort add g "git"
kort add gc "git commit -m '{{message}}'" --global
kort add main "main --branch" --context-lbuffer "^git (checkout|switch) "
kort add TODAY "date +%Y-%m-%d" --evaluate --global
kort add gs "git status --short" --allow-conflict
```

| Flag | Description |
|---|---|
| `--global` | Register as a global abbreviation |
| `--evaluate` | Run expansion as a shell command |
| `--allow-conflict` | Allow conflicts with PATH commands |
| `--context-lbuffer <REGEX>` | Left-buffer regex for context matching |
| `--context-rbuffer <REGEX>` | Right-buffer regex for context matching |
| `--config <PATH>` | Use a custom config file path |

### Interactive

Run `kort add` without arguments to enter interactive mode:

```bash
kort add
```

You will be prompted for the keyword, expansion, type (regular / global / context), and other options.

## Commands

| Command | Description |
|---|---|
| `kort init` | Generate a config template at `~/.config/kort/kort.toml` |
| `kort add` | Add an abbreviation interactively |
| `kort add <keyword> <expansion>` | Add an abbreviation with options |
| `kort compile` | Validate config, detect conflicts, and generate binary cache |
| `kort compile --strict` | Treat suffix conflicts as errors |
| `kort check` | Validate config syntax without compiling |
| `kort list` | Show all registered abbreviations |
| `kort expand` | Expand an abbreviation (called by the zsh widget) |
| `kort next-placeholder` | Jump to next placeholder (called by the zsh widget) |

## Auto-Recompilation

When you edit `kort.toml`, the next expansion automatically detects the stale cache and recompiles. No manual `kort compile` needed after config changes.

## License

MIT
