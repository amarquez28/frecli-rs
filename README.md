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

### Prebuilt binary (no Rust needed)

Each [release](https://github.com/amarquez28/frecli-rs/releases) ships one
archive per platform. Each archive contains the `frecli-rs` binary plus the
`hook.bash` and `hook.zsh` files that match that version.

| Platform | Archive |
|---|---|
| Linux x86_64 | `frecli-rs-x86_64-unknown-linux-musl.tar.gz` |
| Linux arm64 | `frecli-rs-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `frecli-rs-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `frecli-rs-x86_64-apple-darwin.tar.gz` |

The Linux builds are static, so they run on any distribution (Debian, Ubuntu,
Fedora, Arch, Alpine, ...) with nothing else installed.

#### Linux

```sh
# 1. Download and unpack (uname -m prints x86_64 or aarch64)
mkdir frecli && cd frecli
curl -fsSL "https://github.com/amarquez28/frecli-rs/releases/latest/download/frecli-rs-$(uname -m)-unknown-linux-musl.tar.gz" | tar -xz

# 2. Install the binary and your shell's hook (use hook.zsh and ~/.zshrc for zsh)
mkdir -p ~/.local/bin ~/.config/frecli
install -m 755 frecli-rs ~/.local/bin/frecli-rs
cp hook.bash ~/.config/frecli/hook.bash
echo 'source ~/.config/frecli/hook.bash' >> ~/.bashrc

# 3. Start a new shell
exec bash
```

If `fr` or `frecli-rs` isn't found, `~/.local/bin` isn't on your `PATH` yet.
Debian and Ubuntu add it at login when the directory exists, so logging out
and back in is enough; otherwise add
`export PATH="$HOME/.local/bin:$PATH"` to `~/.bashrc` above the frecli line.

#### macOS

Same steps with the macOS archive (use `aarch64-apple-darwin` or
`x86_64-apple-darwin`, since `uname -m` reports `arm64` on Apple Silicon), and
`~/.zshrc` with `hook.zsh` for the default shell. A downloaded binary is
quarantined by macOS; clear it with
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
