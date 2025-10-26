#!/usr/bin/env bash
set -euo pipefail

VERSION="0._._"

TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-pc-windows-gnu"
)

OUTPUT_DIR="$HOME/SHARE"
mkdir -p "$OUTPUT_DIR"

# === CLI ===
CRATE_NAME="boosty_downloader_cli"

# Release
for TARGET in "${TARGETS[@]}"; do
    echo "Building CLI release for $TARGET..."
    cargo build --release --target "$TARGET"

    BIN_PATH="target/$TARGET/release/$CRATE_NAME"
    if [[ "$TARGET" == *"windows"* ]]; then
        BIN_PATH="${BIN_PATH}.exe"
    fi

    if [[ "$TARGET" == *"windows"* ]]; then
        OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-windows-x86_64.exe"
    else
        OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-linux-x86_64"
    fi

    cp "$BIN_PATH" "$OUTPUT_FILE"
    echo "Saved: $OUTPUT_FILE"
done

# Debug
# for TARGET in "${TARGETS[@]}"; do
#     echo "Building debug for $TARGET..."
#     cargo build --target "$TARGET"

#     BIN_PATH="target/$TARGET/debug/$CRATE_NAME"
#     [[ "$TARGET" == *"windows"* ]] && BIN_PATH="${BIN_PATH}.exe"

#     if [[ "$TARGET" == *"windows"* ]]; then
#         OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-windows-x86_64_debug.exe"
#     else
#         OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-linux-x86_64_debug"
#     fi

#     cp "$BIN_PATH" "$OUTPUT_FILE"
#     echo "Saved: $OUTPUT_FILE"
# done

# === GUI ===
CRATE_NAME="boosty_downloader_gui"

# Release
for TARGET in "${TARGETS[@]}"; do
    echo "Building GUI release for $TARGET..."
    cargo build --release --target "$TARGET"

    BIN_PATH="target/$TARGET/release/$CRATE_NAME"
    if [[ "$TARGET" == *"windows"* ]]; then
        BIN_PATH="${BIN_PATH}.exe"
    fi

    if [[ "$TARGET" == *"windows"* ]]; then
        OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-windows-x86_64.exe"
    else
        OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-linux-x86_64"
    fi

    cp "$BIN_PATH" "$OUTPUT_FILE"
    echo "Saved: $OUTPUT_FILE"
done

# Debug
# for TARGET in "${TARGETS[@]}"; do
#     echo "Building debug for $TARGET..."
#     cargo build --target "$TARGET"

#     BIN_PATH="target/$TARGET/debug/$CRATE_NAME"
#     [[ "$TARGET" == *"windows"* ]] && BIN_PATH="${BIN_PATH}.exe"

#     if [[ "$TARGET" == *"windows"* ]]; then
#         OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-windows-x86_64_debug.exe"
#     else
#         OUTPUT_FILE="$OUTPUT_DIR/${CRATE_NAME}-${VERSION}-linux-x86_64_debug"
#     fi

#     cp "$BIN_PATH" "$OUTPUT_FILE"
#     echo "Saved: $OUTPUT_FILE"
# done

echo "All builds finished successfully!"
