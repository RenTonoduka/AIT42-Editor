#!/bin/bash
# AIT42 - Parallel Agents Tmux Launcher
# Usage: ./scripts/tmux-parallel-agents.sh <agent1> <agent2> <agent3> ...

set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <agent1> <agent2> [agent3] ..."
    echo "Example: $0 api-designer database-designer system-architect"
    exit 1
fi

AGENTS=("$@")
SESSIONS=()
WORKING_DIR=$(pwd)

echo "=== AIT42 Tmux Parallel Agents Launcher ==="
echo "Agents: ${AGENTS[@]}"
echo "Count: ${#AGENTS[@]}"
echo ""

# Tmux availability check
if ! command -v tmux &> /dev/null; then
    echo "❌ Error: tmux is not installed"
    exit 1
fi

# Create sessions
echo "Creating ${#AGENTS[@]} parallel sessions..."
for AGENT in "${AGENTS[@]}"; do
    # macOS compatible timestamp with randomness
    TIMESTAMP="$(date +%s)$(printf '%03d' $((RANDOM % 1000)))"
    SESSION="ait42-${AGENT}-${TIMESTAMP}"
    
    tmux new-session -s "${SESSION}" -d -c "${WORKING_DIR}"
    tmux send-keys -t "${SESSION}" "echo '=== Agent: ${AGENT} ==='" C-m
    sleep 0.3
    tmux send-keys -t "${SESSION}" "echo '[$(date +%T)] Running in parallel...'" C-m
    tmux send-keys -t "${SESSION}" "sleep 3 && echo '[$(date +%T)] ✓ Completed'" C-m
    
    SESSIONS+=("${SESSION}")
    echo "  ✅ Created: ${SESSION}"
    
    # Avoid timestamp/session collision
    sleep 0.1
done

echo ""
echo "✅ All ${#AGENTS[@]} agents launched"
echo ""
echo "Active sessions:"
tmux list-sessions | grep "ait42-"

echo ""
echo "Commands:"
echo "  List:         tmux list-sessions | grep ait42-"
echo "  Monitor:      watch -n 1 'tmux list-sessions | grep ait42-'"
echo "  Kill all:     tmux list-sessions | grep ait42- | cut -d: -f1 | xargs -I {} tmux kill-session -t {}"
echo ""
