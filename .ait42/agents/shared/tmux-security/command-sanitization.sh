#!/usr/bin/env bash
# AIT42 Tmux Security Module: Command Sanitization
# Version: 1.0.1
# Purpose: Prevent command injection attacks

set -euo pipefail

# Sanitize command for safe execution
# Args: $1 - command string
# Returns: 0 if safe, 1 if dangerous, stdout: sanitized command
sanitize_command() {
    local cmd="$1"

    # Empty check
    if [[ -z "$cmd" ]]; then
        echo "Error: Command cannot be empty" >&2
        return 1
    fi

    # Dangerous character detection (using case for each pattern)
    # Detects: semicolon, pipe, redirect, command substitution
    case "$cmd" in
        *\;*)
            echo "Error: Command contains semicolon" >&2
            return 1
            ;;
        *\|*)
            echo "Error: Command contains pipe" >&2
            return 1
            ;;
        *\&*)
            echo "Error: Command contains ampersand" >&2
            return 1
            ;;
        *\$*)
            echo "Error: Command contains dollar sign (variable expansion)" >&2
            return 1
            ;;
        *\<*|*\>*)
            echo "Error: Command contains redirect" >&2
            return 1
            ;;
        *\(*)
            echo "Error: Command contains parenthesis (subshell)" >&2
            return 1
            ;;
        *\{*)
            echo "Error: Command contains brace expansion" >&2
            return 1
            ;;
        *\`*)
            echo "Error: Command contains backtick (command substitution)" >&2
            return 1
            ;;
    esac

    echo "$cmd"
    return 0
}

# Safe send-keys wrapper
# Args: $1 - session name, $2 - command
# Returns: 0 if successful, 1 if failed
safe_send_keys() {
    local session_name="$1"
    local command="$2"

    # Source input validation if not already loaded
    if ! declare -f validate_session_name &>/dev/null; then
        source "$(dirname "${BASH_SOURCE[0]}")/input-validation.sh"
    fi

    # Input validation
    validate_session_name "$session_name" || return 1

    # Command sanitization
    local sanitized_cmd
    sanitized_cmd=$(sanitize_command "$command") || return 1

    # Safe command execution (double-quoted)
    tmux send-keys -t "$session_name" "$sanitized_cmd" C-m
    return 0
}

# Validate agent name for tmux session
# Args: $1 - agent name
# Returns: 0 if valid, 1 if invalid
validate_agent_name() {
    local agent_name="$1"

    # Empty check
    if [[ -z "$agent_name" ]]; then
        echo "Error: Agent name cannot be empty" >&2
        return 1
    fi

    # Length check
    if [[ ${#agent_name} -gt 50 ]]; then
        echo "Error: Agent name too long (max 50 chars)" >&2
        return 1
    fi

    # Character validation
    if [[ ! "$agent_name" =~ ^[a-z0-9-]+$ ]]; then
        echo "Error: Agent name must be lowercase with hyphens only" >&2
        return 1
    fi

    return 0
}

# Export functions
export -f sanitize_command
export -f safe_send_keys
export -f validate_agent_name
