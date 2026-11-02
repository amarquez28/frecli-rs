# frecli

A frequency-based command picker for zsh and bash, written in Rust. Works on
macOS and Linux.

frecli quietly logs every command that exits successfully in your shell. Type
`fr` and you get an interactive list of your most-used commands — pick one and
it runs, or paste it onto your prompt to edit first. Think of it as
`history | sort | uniq -c | sort -rn`, but instant and interactive.

```
 Key   │ Command
───────┼──────────────────────────────────────────
 0     │ cargo build --release
 1     │ git status
 2     │ pwd | pbcopy
 ...

 [enter] run   [p] paste   [esc] cancel
```

## Installation

### From source (requires [Rust](https://rustup.rs))

```sh
git clone https://github.com/amarquez28/frecli-rs.git
cd frecli-rs
./scripts/install.sh
```

The install script detects your shell from `$SHELL`, builds the release
binary, copies it to `~/.local/bin/`, copies the matching hook to
`~/.config/frecli/`, and adds a `source` line to your rc file (skipped if one
is already there). Restart your shell afterwards.

| Shell | rc file edited |
|---|---|
| zsh | `~/.zshrc` (or `$ZDOTDIR/.zshrc`) |
| bash (Linux) | `~/.bashrc` |
| bash (macOS) | `~/.bash_profile` |

To pick a shell explicitly: `./scripts/install.sh --shell bash`.

### Prebuilt binary

Download `frecli-rs` for your platform from the
[latest release](https://github.com/amarquez28/frecli-rs/releases), put it
somewhere on your `$PATH` (e.g. `~/.local/bin/`), then install the hook for
your shell (replace `zsh` with `bash` and `~/.zshrc` with `~/.bashrc` as needed):

```sh
mkdir -p ~/.config/frecli
curl -fsSL https://raw.githubusercontent.com/amarquez28/frecli-rs/main/scripts/hook.zsh \
    -o ~/.config/frecli/hook.zsh
echo 'source ~/.config/frecli/hook.zsh' >> ~/.zshrc
```

On macOS, a downloaded binary is quarantined; clear it with
`xattr -d com.apple.quarantine ~/.local/bin/frecli-rs`.

## Usage

| Command | What it does |
|---|---|
| `fr` | Open the interactive picker |
| `fr run --limit 20` | Picker with more rows |
| `fr stats --limit 20` | Print the top commands as a table |
| `fr prune --older-than 30` | Delete history entries older than N days (default 90) |
| `fr --help` | Full CLI help |

### Picker keybinds

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection |
| `0`–`9` | Jump to that row |
| `Enter` | Run the selected command |
| `p` | Paste the command onto your prompt without running it |
| `q` / `Esc` / `Ctrl-C` | Cancel |

### Bash notes

A bash function can't write to your prompt the way zsh can, so bash differs
slightly from zsh:

- `Ctrl-G` opens the picker and puts the chosen command on your prompt, ready
  to edit or run (bash 4+; macOS ships bash 3.2, so install a newer one with
  Homebrew to use it). To rebind it, `export FRECLI_BIND_KEY='\C-f'` before
  the hook is sourced; set it to an empty string to disable the binding.
- `fr` then `p` adds the command to the top of your history instead; press `↑`
  to edit it.

## Configuring the ignore list

Commands you don't want logged are stored in `~/.config/frecli/ignore`
(or `$XDG_CONFIG_HOME/frecli/ignore` if that variable is set).
Manage them from the CLI:

| Command | What it does |
|---|---|
| `fr ignore add <pattern>` | Add a command to the ignore list |
| `fr ignore list` | Print the current ignore list |
| `fr ignore remove <command>` | Remove an entry |

Or edit the file directly, one entry per line:

```
ls
cd
clear
```

Each entry matches on a word boundary: `ls` ignores `ls` and `ls -la`, but not
`lsof`. Blank lines are skipped. Changes take effect immediately — the file is
read on every log.

## How it works

- A shell hook (zsh `preexec`/`precmd`, or bash `PROMPT_COMMAND`) logs each
  command that exits with status 0 to a SQLite database at
  `~/.local/share/frecli/history.db` (or `$XDG_DATA_HOME/frecli/history.db`).
  Logging runs in a detached background process, so your prompt is never
  delayed.
- `fr` invocations themselves are never logged, and a command chosen from the
  picker is logged again on success so its ranking stays accurate.
- The picker communicates its choice back to the shell through a private temp
  file (the hook creates it with `mktemp` and passes its path in
  `FRECLI_CHOICE_FILE`), so "run" executes in *your* shell — aliases,
  functions, and `cd` all work.

## Development

```sh
cargo build --release
export FRECLI_BIN=$PWD/target/release/frecli-rs   # point the hook at a dev build
```

## License

MIT
