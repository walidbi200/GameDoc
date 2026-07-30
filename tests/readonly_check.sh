#!/usr/bin/env bash
set -e

echo "Building release binary..."
cargo build --release

echo "Running read-only verification with unshare..."

# Create a temporary directory for our mock HOME
MOCK_HOME=$(mktemp -d)
MOCK_HOME_MNT="${MOCK_HOME}_mnt"
mkdir -p "$MOCK_HOME_MNT"

# Populate it with some minimal fixtures so the tool has something to look at if needed
mkdir -p "$MOCK_HOME/.steam/steam"

# Run gamedoc inside a namespace where $HOME is mounted read-only
# We test both the default and --share modes
unshare -r -m bash -c "
    mount --bind '$MOCK_HOME' '$MOCK_HOME_MNT' &&
    mount -o remount,ro,bind '$MOCK_HOME_MNT' &&
    echo 'Testing default mode...' &&
    HOME='$MOCK_HOME_MNT' target/release/gamedoc > /dev/null &&
    echo 'Testing --share mode...' &&
    HOME='$MOCK_HOME_MNT' target/release/gamedoc --share > /dev/null
"

# Clean up
rm -rf "$MOCK_HOME" "$MOCK_HOME_MNT"

echo "Read-only verification passed!"
