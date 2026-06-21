#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$WORKSPACE_ROOT/Sources"

ACTION="${1:-build}"

if [ "$ACTION" = "dmg" ]; then
    ACTION="build-installer"
fi

case "$ACTION" in
    build|build-installer|install|clean|uninstall) ;;
    *)
    echo "Usage: $0 [build|build-installer|install|clean|uninstall]"
    echo ""
    echo "Actions:"
    echo "  build           Build the application bundle (.app) (default)"
    echo "  build-installer Build the application bundle and disk image (.dmg)"
    echo "  install         Build and install /Applications/VNKey.app"
    echo "  clean           Remove Cargo, frontend, and copied build artifacts"
    echo "  uninstall       Remove /Applications/VNKey.app"
    exit 1
    ;;
esac

run_privileged() {
    if [ -w /Applications ]; then
        "$@"
    else
        sudo "$@"
    fi
}

if [ "$ACTION" = "uninstall" ]; then
    echo "=== Uninstalling VNKey ==="
    killall VNKey 2>/dev/null || true
    if [ -e /Applications/VNKey.app ]; then
        run_privileged rm -rf /Applications/VNKey.app
    fi
    echo "=== Uninstall finished ==="
    exit 0
fi

if [ "$ACTION" = "clean" ]; then
    echo "=== Cleaning build artifacts ==="
    if [ -d "$TAURI_DIR/src-tauri" ]; then
        cd "$TAURI_DIR/src-tauri"
        cargo clean
    fi
    cd "$TAURI_DIR"
    rm -rf build node_modules .svelte-kit
    rm -rf "$WORKSPACE_ROOT/.build"
    echo "=== Clean finished ==="
    exit 0
fi

echo "=== Building VNKey Tauri for macOS ($ACTION) ==="
command -v npm >/dev/null || { echo "Error: npm is required."; exit 1; }
command -v cargo >/dev/null || { echo "Error: Rust/Cargo is required."; exit 1; }

cd "$TAURI_DIR"
npm ci
npm run check

BUNDLES="app"
if [ "$ACTION" = "build-installer" ]; then
    BUNDLES="app,dmg"
fi

rm -rf "$TAURI_DIR/src-tauri/target/release/bundle/macos/VNKey.app"
if [ "$ACTION" = "build-installer" ]; then
    rm -rf "$TAURI_DIR/src-tauri/target/release/bundle/dmg"
fi
npm run tauri build -- --bundles "$BUNDLES"

APP_PATH="$TAURI_DIR/src-tauri/target/release/bundle/macos/VNKey.app"
if [ ! -d "$APP_PATH" ]; then
    echo "Error: expected application bundle was not produced: $APP_PATH"
    exit 1
fi
codesign --force --deep --sign - "$APP_PATH"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

BUILD_OUT_DIR="$WORKSPACE_ROOT/.build"
mkdir -p "$BUILD_OUT_DIR"
echo "=== Copying macOS build artifacts to $BUILD_OUT_DIR ==="
rm -rf "$BUILD_OUT_DIR/VNKey.app"
cp -R "$APP_PATH" "$BUILD_OUT_DIR/"
if [ "$ACTION" = "build-installer" ]; then
    find "$TAURI_DIR/src-tauri/target/release/bundle/dmg" -maxdepth 1 -type f -name '*.dmg' -exec cp {} "$BUILD_OUT_DIR/" \;
fi

if [ "$ACTION" = "install" ]; then
    echo "=== Installing VNKey.app to /Applications ==="
    echo "Closing currently running VNKey app..."
    killall VNKey 2>/dev/null || true
    sleep 1
    
    echo "Replacing /Applications/VNKey.app..."
    if [ -e /Applications/VNKey.app ]; then
        run_privileged rm -rf /Applications/VNKey.app
    fi
    run_privileged ditto "$APP_PATH" /Applications/VNKey.app
    
    echo "Opening new VNKey app..."
    open /Applications/VNKey.app
    echo "=== Installation finished ==="
fi

echo ""
echo "=== Build finished ==="
echo "Application bundle & artifacts copied to: $BUILD_OUT_DIR"
