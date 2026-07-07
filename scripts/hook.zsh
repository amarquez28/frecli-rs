autoload -Uz add-zsh-hook
add-zsh-hook preexec frecli_preexec
add-zsh-hook precmd frecli_precmd

# Resolved from $PATH by default (install.sh puts it in ~/.local/bin).
# Set FRECLI_BIN to use a different binary, e.g. a dev build:
#   export FRECLI_BIN=~/development/frecli-rs/target/release/frecli-rs
_FRECLI_BIN="${FRECLI_BIN:-frecli-rs}"
_frecli_cmd=""

frecli_preexec() {
    local cmd=$1
    local first_word=${cmd%% *}
    if [[ "$first_word" != "fr" && "$first_word" != frecli* ]]; then
        _frecli_cmd=$cmd
    fi
}

frecli_precmd() {
    local exit_code=$?
    if [[ $exit_code -eq 0 && -n "$_frecli_cmd" ]]; then
        ("$_FRECLI_BIN" log "$_frecli_cmd" &) 2>/dev/null
    fi
    _frecli_cmd=""
}

fr() {
    # Non-run arguments pass straight through to the binary:
    # fr --help, fr stats --limit 20, fr prune --older-than 30, ...
    # `fr` and `fr run ...` fall through to the interactive picker below.
    if (( $# > 0 )) && [[ "$1" != "run" ]]; then
        "$_FRECLI_BIN" "$@"
        return $?
    fi

    local choice_file="/tmp/frecli-rs_choice"
    rm -f "$choice_file"
    "$_FRECLI_BIN" "${@:-run}"
    [[ -f "$choice_file" ]] || return 0

    local content
    content=$(<"$choice_file")
    rm -f "$choice_file"
    [[ -n "$content" ]] || return 0

    local prefix="${content%%:*}"
    local cmd="${content#*:}"
    if [[ "$prefix" == "RUN" ]]; then
        # Echo the command so it's obvious what is running (bare `cat`/`bat`
        # would otherwise sit silently waiting on stdin).
        print -r -- "> $cmd"
        eval "$cmd"
        local rc=$?
        # preexec skips the `fr` invocation itself, so log the chosen
        # command here to keep its frecency count accurate.
        if (( rc == 0 )); then
            ("$_FRECLI_BIN" log "$cmd" &) 2>/dev/null
        fi
        return $rc
    elif [[ "$prefix" == "PASTE" ]]; then
        print -z -- "$cmd"
    fi
}
