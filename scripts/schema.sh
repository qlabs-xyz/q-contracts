#!/usr/bin/env sh
set -e

base_dir=$(pwd)

for contract in contracts/*; do
    if [ -d "$contract" ]; then
        echo "--- Generating schema for $contract ---"
        cd "$contract"
        cargo schema
        cd "$base_dir"
    fi
done
