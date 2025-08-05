#!/bin/bash

# Cleanup script for stuck backend processes on port 8080
# Usage: ./cleanup-port.sh [port]

PORT=${1:-8080}

echo "🔍 Checking for processes on port $PORT..."

# Find processes using the port
PIDS=$(lsof -ti:$PORT 2>/dev/null)

if [ -z "$PIDS" ]; then
    echo "✅ No processes found on port $PORT"
    exit 0
fi

echo "🚨 Found processes on port $PORT:"
lsof -i:$PORT

echo ""
echo "🛑 Killing processes..."

# Kill processes gracefully first (SIGTERM)
for PID in $PIDS; do
    echo "  Sending SIGTERM to PID $PID"
    kill -TERM $PID 2>/dev/null
done

# Wait a bit for graceful shutdown
sleep 2

# Check if any processes are still running
REMAINING_PIDS=$(lsof -ti:$PORT 2>/dev/null)

if [ -n "$REMAINING_PIDS" ]; then
    echo "⚠️  Some processes still running, forcing kill (SIGKILL)..."
    for PID in $REMAINING_PIDS; do
        echo "  Sending SIGKILL to PID $PID"
        kill -KILL $PID 2>/dev/null
    done
    sleep 1
fi

# Final check
FINAL_CHECK=$(lsof -ti:$PORT 2>/dev/null)
if [ -z "$FINAL_CHECK" ]; then
    echo "✅ Port $PORT is now free"
else
    echo "❌ Failed to free port $PORT"
    exit 1
fi