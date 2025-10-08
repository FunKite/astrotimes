#!/bin/bash
# Astrotimes macOS Installation Script

set -e

echo "==================================="
echo "Astrotimes v0.1.0 Installation"
echo "==================================="
echo ""

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "Error: This script is for macOS only."
    exit 1
fi

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" != "arm64" ]]; then
    echo "Warning: This binary is built for Apple Silicon (ARM64)."
    echo "Your system is: $ARCH"
    echo "You may need to build from source for Intel Macs."
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Determine installation location
if [[ -w "/usr/local/bin" ]]; then
    INSTALL_DIR="/usr/local/bin"
    echo "Installing to: /usr/local/bin"
elif [[ -d "/opt/homebrew/bin" && -w "/opt/homebrew/bin" ]]; then
    INSTALL_DIR="/opt/homebrew/bin"
    echo "Installing to: /opt/homebrew/bin"
else
    INSTALL_DIR="$HOME/bin"
    echo "Installing to: $HOME/bin"
    mkdir -p "$INSTALL_DIR"

    # Add to PATH if needed
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        echo "Adding $INSTALL_DIR to PATH..."
        if [[ -f "$HOME/.zshrc" ]]; then
            echo 'export PATH="$HOME/bin:$PATH"' >> "$HOME/.zshrc"
            echo "Added to ~/.zshrc"
        fi
        if [[ -f "$HOME/.bash_profile" ]]; then
            echo 'export PATH="$HOME/bin:$PATH"' >> "$HOME/.bash_profile"
            echo "Added to ~/.bash_profile"
        fi
    fi
fi

# Extract and install
echo ""
echo "Extracting astrotimes..."
tar -xzf astrotimes-v0.1.0-macos-arm64.tar.gz

echo "Installing binary..."
cp astrotimes-macos/astrotimes "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/astrotimes"

# Clean up
rm -rf astrotimes-macos

echo ""
echo "==================================="
echo "✅ Installation complete!"
echo "==================================="
echo ""
echo "Location: $INSTALL_DIR/astrotimes"
echo ""
echo "Try it now:"
echo "  astrotimes --city \"New York\""
echo ""
echo "For help:"
echo "  astrotimes --help"
echo ""
echo "If command not found, restart your terminal or run:"
echo "  source ~/.zshrc"
echo ""
