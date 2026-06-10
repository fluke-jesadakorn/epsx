#!/bin/bash

# Configuration
REMOTE_HOST="100.81.224.19"
# Default remote directory. Change this if your project is elsewhere on the server.
REMOTE_DIR="~/epsx" 
# Local directory (current directory)
LOCAL_DIR="$(pwd)/"

# Exclusion list
EXCLUDES=(
  "--exclude=.git/"
  "--exclude=node_modules/"
  "--exclude=target/"
  "--exclude=.next/"
  "--exclude=.turbo/"
  "--exclude=dist/"
  "--exclude=.DS_Store"
  "--exclude=.env*"
)

# Check for fswatch
if ! command -v fswatch &> /dev/null; then
    echo "Error: fswatch is not installed. Please install it using 'brew install fswatch'."
    exit 1
fi

echo "Syncing from $LOCAL_DIR to $REMOTE_HOST:$REMOTE_DIR"
echo "Watching for changes..."

# Initial sync
rsync -avz --delete "${EXCLUDES[@]}" "$LOCAL_DIR" "$REMOTE_HOST:$REMOTE_DIR"

# Watch for changes
fswatch -o . "${EXCLUDES[@]}" | while read f; do
    echo "Change detected. Syncing..."
    rsync -avz --delete "${EXCLUDES[@]}" "$LOCAL_DIR" "$REMOTE_HOST:$REMOTE_DIR"
done
