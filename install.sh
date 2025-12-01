#!/bin/bash

set -e

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "Installing theme-picker..."

TEMP_DIR=$(mktemp -d)
trap 'rm -rf $TEMP_DIR' EXIT

# Download theme_picker
echo "Downloading theme_picker..."
curl -sSL https://github.com/FrederikNorlyk/rust-theme-picker-tui/releases/latest/download/theme_picker -o "$TEMP_DIR/theme_picker"
chmod +x "$TEMP_DIR/theme_picker"
sudo mv "$TEMP_DIR/theme_picker" /usr/local/bin/theme_picker

# Download cli
echo "Downloading norlyk..."
curl -sSL https://github.com/FrederikNorlyk/rust-theme-picker-tui/releases/latest/download/norlyk -o "$TEMP_DIR/norlyk"
chmod +x "$TEMP_DIR/norlyk"
sudo mv "$TEMP_DIR/norlyk" /usr/local/bin/norlyk

echo -e "${GREEN}Installation complete!${NC}"
