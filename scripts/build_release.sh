#!/usr/bin/env bash
set -euo pipefail

TARGETS=("x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu")
OUTPUT_DIR="$HOME/SHARE"
PREFIX="boosty_downloader"
mkdir -p "$OUTPUT_DIR"

get_version() {
    local crate_dir=$1
    cargo pkgid --manifest-path "$PREFIX/$crate_dir/Cargo.toml" | cut -d# -f2 | cut -d: -f2 | cut -d@ -f2
}

build_crate() {
    local crate_name=$1
    local crate_dir=$2
    local version=$3

    for TARGET in "${TARGETS[@]}"; do
        echo "Building release for $crate_name on $TARGET..."
        cargo build --release --target "$TARGET" --manifest-path "$PREFIX/$crate_dir/Cargo.toml"

        BIN_PATH="target/$TARGET/release/$crate_name"
        [[ "$TARGET" == *"windows"* ]] && BIN_PATH="${BIN_PATH}.exe"

        if [[ "$TARGET" == *"windows"* ]]; then
            OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-windows-x86_64.exe"
        else
            OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-linux-x86_64"
        fi

        cp "$BIN_PATH" "$OUTPUT_FILE"
        echo "Saved: $OUTPUT_FILE"
    done

    for TARGET in "${TARGETS[@]}"; do
        echo "Building debug for $crate_name on $TARGET..."
        cargo build --target "$TARGET" --manifest-path "$PREFIX/$crate_dir/Cargo.toml"

        BIN_PATH="target/$TARGET/debug/$crate_name"
        [[ "$TARGET" == *"windows"* ]] && BIN_PATH="${BIN_PATH}.exe"

        if [[ "$TARGET" == *"windows"* ]]; then
            OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-windows-x86_64_debug.exe"
        else
            OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-linux-x86_64_debug"
        fi

        cp "$BIN_PATH" "$OUTPUT_FILE"
        echo "Saved: $OUTPUT_FILE"
    done
}

# === CLI ===
CLI_VERSION=$(get_version "core")
build_crate "boosty_downloader_cli" "core" "$CLI_VERSION"

# === GUI ===
GUI_VERSION=$(get_version "gui")
build_crate "boosty_downloader_gui" "gui" "$GUI_VERSION"

echo "All builds finished successfully!"
