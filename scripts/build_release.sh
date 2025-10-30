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
    local is_tauri=${4:-false}

    for TARGET in "${TARGETS[@]}"; do
        echo "Building release for $crate_name on $TARGET..."

        if [ "$is_tauri" = true ]; then
            cargo tauri build --target "$TARGET"
            BIN_PATH="target/$TARGET/release/$crate_name"
        else
            cargo build --release --target "$TARGET" --manifest-path "$PREFIX/$crate_dir/Cargo.toml"
            BIN_PATH="target/$TARGET/release/$crate_name"
        fi

        [[ "$TARGET" == *"windows"* ]] && BIN_PATH="${BIN_PATH}.exe"

        if [[ "$TARGET" == *"windows"* ]]; then
            OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-windows-x86_64.exe"
        else
            OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-linux-x86_64"
        fi

        cp "$BIN_PATH" "$OUTPUT_FILE"
        echo "Saved: $OUTPUT_FILE"

        if [ "$is_tauri" = true ] && [[ "$TARGET" == *"windows"* ]]; then
            WEBVIEW_DLL="$PREFIX/frontend/webview2-fixed/WebView2Loader.dll"
            ZIP_NAME="$OUTPUT_DIR/${crate_name}-${version}-windows-x86_64.zip"

            if [ -f "$WEBVIEW_DLL" ]; then
                echo "Packing $crate_name + WebView2Loader.dll into $ZIP_NAME"
                (
                    cd "$OUTPUT_DIR"
                    zip -j "$ZIP_NAME" \
                        "${crate_name}-${version}-windows-x86_64.exe" \
                        "$WEBVIEW_DLL"
                )
                echo "Created archive: $ZIP_NAME"
            else
                echo "Warning: $WEBVIEW_DLL not found, skipping DLL packaging"
            fi
        fi
    done

    # for TARGET in "${TARGETS[@]}"; do
    #     echo "Building debug for $crate_name on $TARGET..."

    #     if [ "$is_tauri" = true ]; then
    #         cargo tauri build --target "$TARGET" --manifest-path "$PREFIX/$crate_dir/Cargo.toml"
    #         BIN_PATH="target/$TARGET/debug/$crate_name"
    #     else
    #         cargo build --target "$TARGET" --manifest-path "$PREFIX/$crate_dir/Cargo.toml"
    #         BIN_PATH="target/$TARGET/debug/$crate_name"
    #     fi
        
    #     [[ "$TARGET" == *"windows"* ]] && BIN_PATH="${BIN_PATH}.exe"

    #     if [[ "$TARGET" == *"windows"* ]]; then
    #         OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-windows-x86_64_debug.exe"
    #     else
    #         OUTPUT_FILE="$OUTPUT_DIR/${crate_name}-${version}-linux-x86_64_debug"
    #     fi

    #     cp "$BIN_PATH" "$OUTPUT_FILE"
    #     echo "Saved: $OUTPUT_FILE"
    # done
}

# === CLI ===
CLI_VERSION=$(get_version "core")
build_crate "boosty_downloader_cli" "core" "$CLI_VERSION"

# === GUI ===
GUI_VERSION=$(get_version "src-tauri")
build_crate "boosty_downloader_gui" "src-tauri" "$GUI_VERSION" true

echo "All builds finished successfully!"
