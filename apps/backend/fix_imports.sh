#!/bin/bash

# Fix imports for all Rust files that need them

echo "Adding missing imports..."

# Find all .rs files and add necessary imports
find src -name "*.rs" -type f | while read -r file; do
    echo "Processing: $file"
    
    # Check if the file uses Uuid but doesn't import it
    if grep -q "Uuid::\|: Uuid\|pub.*Uuid" "$file" && ! grep -q "use uuid::Uuid" "$file"; then
        echo "  Adding uuid::Uuid to $file"
        sed -i '' '1a\
use uuid::Uuid;
' "$file"
    fi
    
    # Check if the file uses DateTime but doesn't import it
    if grep -q "DateTime<\|DateTime::" "$file" && ! grep -q "use chrono.*DateTime" "$file"; then
        echo "  Adding chrono imports to $file"
        sed -i '' '1a\
use chrono::{DateTime, Utc};
' "$file"
    fi
    
    # Check if the file uses IpAddr but doesn't import it
    if grep -q "IpAddr" "$file" && ! grep -q "use std::net::IpAddr" "$file"; then
        echo "  Adding IpAddr to $file"
        sed -i '' '1a\
use std::net::IpAddr;
' "$file"
    fi
done

echo "Cleaning up duplicates..."

# Remove duplicate imports
find src -name "*.rs" -type f -exec sed -i '' '
# Remove duplicate uuid imports except first
/use uuid::Uuid;/{
N
s/use uuid::Uuid;\nuse uuid::Uuid;//
}
# Remove duplicate chrono imports except first  
/use chrono::{DateTime, Utc};/{
N
s/use chrono::{DateTime, Utc};\nuse chrono::{DateTime, Utc};//
}
# Remove duplicate IpAddr imports except first
/use std::net::IpAddr;/{
N  
s/use std::net::IpAddr;\nuse std::net::IpAddr;//
}
' {} \;

echo "Done fixing imports!"