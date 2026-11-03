# frecli hook for bash. Source this from ~/.bashrc (install.sh does it for you).
[[ $- == *i* ]] || return 0

# Resolved from $PATH by default (install.sh puts it in ~/.local/bin).
# Set FRECLI_BIN to use a different binary, e.g. a dev build:
#   export FRECLI_BIN=~/development/frecli-rs/target/release/frecli-rs
_FRECLI_BIN="${FRECLI_BIN:-frecli-rs}"

# History number of the last command we looked at. PROMPT_COMMAND also runs
# when you just press Enter, and `history 1` would return the same command
# again, so this stops it being logged twice.
__frecli_last_hist=""
__frecli_init_hist() {
    local num _
    read -r num _ <<<"$(HISTTIMEFORMAT= history 1)"
    __frecli_last_hist=$num
}
__frecli_init_hist

__frecli_log() {
    local exit_code=$?
    local num cmd
    read -r num cmd <<<"$(HISTTIMEFORMAT= history 1)"
    [[ -n "$num" && "$num" != "$__frecli_last_hist" ]] || return $exit_code
    __frecli_last_hist=$num

    local first_word=${cmd%% *}
    if [[ $exit_code -eq 0 && -n "$cmd" && "$first_word" != "fr" && "$first_word" != frecli* ]]; then
        ("$_FRECLI_BIN" log "$cmd" &) 2>/dev/null
    fi
    return $exit_code
}

# Run first so $? is still the exit status of the user's command.
if [[ "$(declare -p PROMPT_COMMAND 2>/dev/null)" == "declare -a"* ]]; then
    [[ " ${PROMPT_COMMAND[*]} " == *" __frecli_log "* ]] ||
        PROMPT_COMMAND=(__frecli_log "${PROMPT_COMMAND[@]}")
else
    [[ "$PROMPT_COMMAND" == *__frecli_log* ]] ||
        PROMPT_COMMAND="__frecli_log${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi

# Runs the picker and leaves the raw "RUN:cmd" / "PASTE:cmd" choice (if any)
# in __frecli_choice. It can't print it instead: the picker draws its UI on
# stdout, so the call must not be wrapped in $(...).
__frecli_pick() {
    local choice_file
    choice_file=$(mktemp "${TMPDIR:-/tmp}/frecli-choice.XXXXXX") || return 1
    FRECLI_CHOICE_FILE="$choice_file" "$_FRECLI_BIN" "$@"
    __frecli_choice=$(<"$choice_file")
    rm -f "$choice_file"
}

fr() {
    # Non-run arguments pass straight through to the binary:
    # fr --help, fr stats --limit 20, fr prune --older-than 30, ...
    # `fr` and `fr run ...` fall through to the interactive picker below.
    if (($# > 0)) && [[ "$1" != "run" ]]; then
        "$_FRECLI_BIN" "$@"
        return $?
    fi

    __frecli_pick "${@:-run}" || return 1
    local content=$__frecli_choice
    [[ -n "$content" ]] || return 0

    local prefix="${content%%:*}"
    local cmd="${content#*:}"
    if [[ "$prefix" == "RUN" ]]; then
        # Echo the command so it's obvious what is running (bare `cat`/`bat`
        # would otherwise sit silently waiting on stdin).
        printf '> %s\n' "$cmd"
        history -s -- "$cmd"
        eval "$cmd"
        local rc=$?
        # The `fr` invocation itself is skipped by __frecli_log, so log the
        # chosen command here to keep its frecency count accurate. (The
        # history entry above is skipped too, as it matches the same number.)
        if ((rc == 0)); then
            ("$_FRECLI_BIN" log "$cmd" &) 2>/dev/null
        fi
        __frecli_init_hist
        return $rc
    elif [[ "$prefix" == "PASTE" ]]; then
        # A bash function can't write to the prompt (zsh's `print -z`), so
        # the closest thing is putting it at the top of history. For true
        # paste-on-prompt use the Ctrl-G key binding below.
        history -s -- "$cmd"
        __frecli_init_hist
        printf 'Added to history — press ↑ to edit: %s\n' "$cmd"
    fi
}

# Ctrl-G opens the picker and puts the chosen command on the prompt, ready to
# edit or run. Needs bash 4+ (READLINE_LINE); set FRECLI_BIND_KEY to another
# readline key sequence to rebind it, or to an empty string to disable.
__frecli_widget() {
    __frecli_pick run || return 0
    local content=$__frecli_choice
    [[ -n "$content" ]] || return 0
    local cmd="${content#*:}"
    READLINE_LINE="${READLINE_LINE:0:READLINE_POINT}${cmd}${READLINE_LINE:READLINE_POINT}"
    READLINE_POINT=$((READLINE_POINT + ${#cmd}))
}
if ((BASH_VERSINFO[0] >= 4)); then
    _frecli_key="${FRECLI_BIND_KEY-\\C-g}"
    [[ -z "$_frecli_key" ]] || bind -x "\"$_frecli_key\": __frecli_widget"
    unset _frecli_key
fi
