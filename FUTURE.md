# frecli-rs — Future Updates & Features

## Distribution
- [ ] Homebrew formula so users can `brew install frecli-rs` instead of building from source
- [ ] Pre-built binaries for Linux (x86_64, aarch64) in addition to macOS releases
- [ ] AUR package for Arch Linux users

## Shell Support
- [ ] Fish shell hook (`fish_preexec` / `fish_postexec` equivalents)
- [ ] Bash hook (`PROMPT_COMMAND` based)
- [ ] Generic install script that detects shell and sources the right hook

## Configuration
- [ ] Config file support (`~/.config/frecli/config.toml`) for:
  - Custom keybinds for run/paste/cancel
  - Default limit for `stats` and `run`
  - Custom DB and ignore list paths
  - Theme/color options
  - Ability to pin commands and command snippets
- [ ] `frecli config` subcommand to view/edit config interactively

## TUI Improvements
- [ ] Search/filter bar inside the `run` picker (type to filter commands)
- [ ] Mouse support for clicking rows
- [ ] Scrolling for when command history exceeds terminal height
- [ ] Show command frequency count alongside each entry in the picker
- [ ] Configurable column widths based on terminal size (`crossterm::terminal::size()`)

## Stats Improvements
- [ ] Sparkline showing command frequency over time
- [ ] Filter stats by recency (`--since 7d`, `--since 30d`)
- [ ] Export stats to CSV (`frecli stats --export`)
- [ ] `frecli stats --json` for machine-readable output

## DB & Data
- [ ] `frecli import` — import history from shell history file (`.zsh_history`, `.bash_history`)
- [ ] `frecli export` — dump DB to plain text or JSON
- [ ] DB migrations system for schema changes across versions
- [ ] Optional deduplication window — don't log the same command twice within N seconds

## Quality of Life
- [x] `frecli ignore add <pattern>` — add to ignore list from CLI instead of editing the file manually
- [x] `frecli ignore list` — print current ignore list
- [x] `frecli ignore remove <pattern>` — remove the pattern from the ignore list
- [ ] `frecli clear` — wipe the entire history DB
- [ ] Man page generation from clap (`clap_mangen`)
- [ ] Shell completions generation (`clap_complete`) for zsh/bash/fish
