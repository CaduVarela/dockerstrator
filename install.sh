#!/bin/bash

# Dockerstrator Installation Script
# Simple one-liner install: curl -sSL https://raw.githubusercontent.com/caduvarela/dockerstrator/master/install.sh | sh

set -e

REPO_OWNER="caduvarela"
REPO_NAME="dockerstrator"
INSTALL_PATH="${HOME}/.local/bin"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}Installing Dockerstrator...${NC}\n"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="dockerstrator-linux-x64"
        elif [ "$ARCH" = "aarch64" ]; then
            BINARY_NAME="dockerstrator-linux-arm64"
        else
            echo -e "${RED}Unsupported architecture: $ARCH${NC}"
            exit 1
        fi
        ;;
    Darwin*)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="dockerstrator-macos-x64"
        elif [ "$ARCH" = "arm64" ]; then
            BINARY_NAME="dockerstrator-macos-arm64"
        else
            echo -e "${RED}Unsupported architecture: $ARCH${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_PATH"

# Download latest release
RELEASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$BINARY_NAME"

echo -e "${BLUE}Downloading from: $RELEASE_URL${NC}"
if ! command -v curl >/dev/null 2>&1; then
    echo -e "${RED}curl is required but not installed${NC}"
    exit 1
fi

TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

if ! curl -sSfL "$RELEASE_URL" -o "$TEMP_FILE"; then
    echo -e "${RED}Failed to download. Make sure:${NC}"
    echo "  1. The repository exists and is public"
    echo "  2. A release with $BINARY_NAME has been created"
    echo "  3. You have internet connection"
    exit 1
fi

# Move to install location
mv "$TEMP_FILE" "$INSTALL_PATH/dockerstrator"
chmod +x "$INSTALL_PATH/dockerstrator"

# Check if .local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_PATH:"* ]]; then
    echo -e "${BLUE}Adding $INSTALL_PATH to PATH...${NC}"

    # Detect shell
    SHELL_RC=""
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    fi

    if [ -n "$SHELL_RC" ]; then
        if ! grep -q "$INSTALL_PATH" "$SHELL_RC"; then
            echo "export PATH=\"$INSTALL_PATH:\$PATH\"" >> "$SHELL_RC"
            echo -e "${BLUE}Added to $SHELL_RC${NC}"
        fi
    fi
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Usage:"
echo "  dockerstrator"
echo ""
echo "If you get 'command not found', reload your shell:"
echo "  source ~/.bashrc  (or source ~/.zshrc)"
echo ""
