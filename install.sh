#!/usr/bin/env bash
set -e

APP_ID="com.marcel.system-logs-diagnostic"
BINARY_NAME="system-logs-diagnostic"
BINARY_PATH="./target/release/$BINARY_NAME"

INSTALL_DIR="/usr/local/bin"
DESKTOP_ENTRY_DIR="/usr/share/applications"

echo "Installing $BINARY_NAME..."

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH."
    exit 1
fi

sudo cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

# default icon
if [ -f "com.marcel.system-logs-diagnostic.png" ]; then
    sudo mkdir -p /usr/share/pixmaps
    sudo cp \
        com.marcel.system-logs-diagnostic.png \
        /usr/share/pixmaps/com.marcel.system-logs-diagnostic.png
fi

# hicolor icons
for size in 48x48 64x64 96x96 128x128 256x256 512x512; do
    ICON_SOURCE="icons/hicolor/$size/apps/$APP_ID.png"
    ICON_DEST="/usr/share/icons/hicolor/$size/apps"

    if [ -f "$ICON_SOURCE" ]; then
        sudo mkdir -p "$ICON_DEST"
        sudo cp "$ICON_SOURCE" "$ICON_DEST/$APP_ID.png"
    fi
done

sudo gtk-update-icon-cache -f /usr/share/icons/hicolor/ || true

cat <<EOF | sudo tee "$DESKTOP_ENTRY_DIR/$APP_ID.desktop" > /dev/null
[Desktop Entry]
Name=System Diagnostic
Comment=Analyze system logs and kernel panics
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=$APP_ID
Terminal=false
Type=Application
Categories=System;Monitor;GTK;
StartupWMClass=$BINARY_NAME
Keywords=log;diagnostic;kernel;panic;
EOF

# LICENSE install
if [ -f "LICENSE" ]; then
    sudo mkdir -p /usr/share/licenses/$BINARY_NAME
    sudo cp LICENSE /usr/share/licenses/$BINARY_NAME/LICENSE
fi

sudo update-desktop-database "$DESKTOP_ENTRY_DIR"

echo "Installation complete."
