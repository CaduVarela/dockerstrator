#!/bin/sh

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

printf "${BLUE}Installing Dockerstrator...${NC}\n\n"

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
            printf "${RED}Unsupported architecture: $ARCH${NC}\n"
            exit 1
        fi
        ;;
    Darwin*)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="dockerstrator-macos-x64"
        elif [ "$ARCH" = "arm64" ]; then
            BINARY_NAME="dockerstrator-macos-arm64"
        else
            printf "${RED}Unsupported architecture: $ARCH${NC}\n"
            exit 1
        fi
        ;;
    *)
        printf "${RED}Unsupported OS: $OS${NC}\n"
        exit 1
        ;;
esac

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_PATH"

# Download latest release
RELEASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$BINARY_NAME"

printf "${BLUE}Downloading from: $RELEASE_URL${NC}\n"
if ! command -v curl >/dev/null 2>&1; then
    printf "${RED}curl is required but not installed${NC}\n"
    exit 1
fi

TEMP_FILE=$(mktemp)
trap 'rm -f "$TEMP_FILE"' EXIT

if ! curl -sSfL "$RELEASE_URL" -o "$TEMP_FILE"; then
    printf "${RED}Failed to download. Make sure:${NC}\n"
    echo "  1. The repository exists and is public"
    echo "  2. A release with $BINARY_NAME has been created"
    echo "  3. You have internet connection"
    exit 1
fi

# Move to install location
mv "$TEMP_FILE" "$INSTALL_PATH/dockerstrator"
chmod +x "$INSTALL_PATH/dockerstrator"

# Check if .local/bin is in PATH
case ":$PATH:" in
    *":$INSTALL_PATH:"*) ;;
    *)
        printf "${BLUE}Adding $INSTALL_PATH to PATH...${NC}\n"

        SHELL_RC=""
        if [ -f "$HOME/.bashrc" ]; then
            SHELL_RC="$HOME/.bashrc"
        elif [ -f "$HOME/.zshrc" ]; then
            SHELL_RC="$HOME/.zshrc"
        fi

        if [ -n "$SHELL_RC" ]; then
            if ! grep -q "$INSTALL_PATH" "$SHELL_RC"; then
                echo "export PATH=\"$INSTALL_PATH:\$PATH\"" >> "$SHELL_RC"
                printf "${BLUE}Added to $SHELL_RC${NC}\n"
            fi
        fi
        ;;
esac

echo ""
printf "${GREEN}Installation complete!${NC}\n"
echo ""
echo "Usage:"
echo "  dockerstrator"
echo ""
echo "If you get 'command not found', reload your shell:"
echo "  source ~/.bashrc  (or source ~/.zshrc)"
echo ""
