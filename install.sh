#!/bin/bash
set -e

# TasQ Installation Script
echo "🚀 Installing TasQ..."

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "📋 Detected: $OS-$ARCH"

# For now, we'll build from source since we don't have releases yet
echo "🔨 Building from source..."

# Check if Rust is installed
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if git is installed
if ! command -v git >/dev/null 2>&1; then
    echo "❌ Git is not installed. Please install Git first."
    exit 1
fi

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Clone repository
echo "📥 Cloning repository..."
git clone https://github.com/arcalumis/tasq.git
cd tasq

# Build release binary
echo "🔨 Building release binary..."
cargo build --release

# Install to system
BINARY_PATH="target/release/tasq"

if [ -w "/usr/local/bin" ]; then
    cp "$BINARY_PATH" /usr/local/bin/
    echo "✅ TasQ installed to /usr/local/bin/tasq"
elif [ -w "$HOME/.local/bin" ]; then
    mkdir -p "$HOME/.local/bin"
    cp "$BINARY_PATH" "$HOME/.local/bin/"
    echo "✅ TasQ installed to $HOME/.local/bin/tasq"
    echo "📝 Make sure $HOME/.local/bin is in your PATH"
else
    echo "🔐 Installing to /usr/local/bin (requires sudo)..."
    sudo cp "$BINARY_PATH" /usr/local/bin/
    echo "✅ TasQ installed to /usr/local/bin/tasq"
fi

# Cleanup
cd /
rm -rf "$TEMP_DIR"

# Verify installation
if command -v tasq >/dev/null 2>&1; then
    echo "🎉 Installation successful!"
    echo "📖 Run 'tasq --help' to get started"
    echo "🚀 Run 'tasq' for interactive mode"
else
    echo "❌ Installation failed. Please check your PATH"
    exit 1
fi