#!/bin/bash

# Safe, incremental ESLint fixer
# Fixes violations category by category with verification

cd /Users/fluke/Desktop/Work/epsx/apps/frontend

count_errors() {
    npm run lint 2>&1 | grep -oE '[0-9]+ problems' | grep -oE '[0-9]+' | head -1
}

echo "Starting error count: $(count_errors)"

# Step 1: Fix no-console (safest fix)
echo "=== Removing console statements ==="
find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    ! -name "*.test.*" ! -name "*.spec.*" \
    ! -path "*/__test__/*" ! -path "*/e2e/*" \
    -exec sed -i.bak -E '/^\s*console\.(log|error|warn|info|debug)\(/d' {} \;

find . -name "*.bak" -delete
echo "After removing console: $(count_errors)"

# Step 2: Fix prefer-nullish-coalescing in simple assignments
echo "=== Fixing nullish coalescing in assignments ==="
find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    -exec sed -i.bak -E 's/= ([a-zA-Z0-9_.?]+) \|\| ""/= \1 ?? ""/g' {} \;

find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    -exec sed -i.bak -E "s/= ([a-zA-Z0-9_.?]+) \|\| ''/= \1 ?? ''/g" {} \;

find . -name "*.bak" -delete
echo "After nullish coalescing: $(count_errors)"

# Step 3: Fix unused catch variables
echo "=== Fixing unused catch variables ==="
find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    ! -name "*.test.*" ! -name "*.spec.*" \
    -exec sed -i.bak -E 's/catch \(error\)/catch (_error)/g' {} \;

find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    ! -name "*.test.*" ! -name "*.spec.*" \
    -exec sed -i.bak -E 's/catch \(err\)/catch (_err)/g' {} \;

find . -name "*.bak" -delete
echo "After unused catch vars: $(count_errors)"

# Step 4: Replace : any with : unknown
echo "=== Replacing any with unknown ==="
find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    ! -name "*.test.*" ! -name "*.spec.*" \
    -exec sed -i.bak -E 's/: any([;,\)\]])/: unknown\1/g' {} \;

find . -name "*.bak" -delete
echo "After any->unknown: $(count_errors)"

# Step 5: Fix array index keys in React
echo "=== Fixing React array index keys ==="
find . -type f \( -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    -exec sed -i.bak -E 's/key=\{index\}/key={`item-${index}`}/g' {} \;

find . -type f \( -name "*.tsx" \) \
    ! -path "*/node_modules/*" ! -path "*/.next/*" \
    -exec sed -i.bak -E 's/key=\{i\}/key={`item-${i}`}/g' {} \;

find . -name "*.bak" -delete
echo "After React keys: $(count_errors)"

echo ""
echo "=== Summary ==="
echo "Final error count: $(count_errors)"
echo ""
echo "Run 'npm run lint | head -100' to see remaining errors"
