#!/bin/bash
# AIT42 - Single Agent Tmux Launcher
# Usage: ./scripts/tmux-single-agent.sh <agent-name> <task-description>

set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <agent-name> <task-description>"
    echo "Example: $0 backend-developer 'Implement user authentication API'"
    exit 1
fi

AGENT="$1"
TASK="$2"
# macOS compatible timestamp (seconds + random for uniqueness)
TIMESTAMP="$(date +%s)$(printf '%03d' $((RANDOM % 1000)))"
SESSION="ait42-${AGENT}-${TIMESTAMP}"
WORKING_DIR=$(pwd)

echo "=== AIT42 Tmux Single Agent Launcher ==="
echo "Agent: ${AGENT}"
echo "Task: ${TASK}"
echo "Session: ${SESSION}"
echo ""

# Tmux availability check
if ! command -v tmux &> /dev/null; then
    echo "❌ Error: tmux is not installed"
    echo "Install: brew install tmux (macOS) or apt install tmux (Linux)"
    exit 1
fi

# Create session
echo "Creating tmux session..."
tmux new-session -s "${SESSION}" -d -c "${WORKING_DIR}"
echo "✅ Session created: ${SESSION}"

# Send task
echo "Sending task to agent..."
tmux send-keys -t "${SESSION}" "echo '=== ${AGENT} ===' && echo 'Task: ${TASK}'" C-m
sleep 0.5
tmux send-keys -t "${SESSION}" "echo '[$(date +%T)] Agent starting...'" C-m

# Note: In production, you would invoke the actual agent here
tmux send-keys -t "${SESSION}" "echo '[$(date +%T)] Processing task...'" C-m
tmux send-keys -t "${SESSION}" "sleep 2" C-m
tmux send-keys -t "${SESSION}" "echo '[$(date +%T)] ✓ Task completed'" C-m

echo ""
echo "✅ Agent launched in tmux session"
echo ""
echo "Commands:"
echo "  View output:  tmux capture-pane -t ${SESSION} -p"
echo "  Attach:       tmux attach -t ${SESSION}"
echo "  Kill:         tmux kill-session -t ${SESSION}"
echo ""
