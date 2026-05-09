APP_ID="com.marcel.system-logs-diagnostic"
BINARY_NAME="system-logs-diagnostic"
BINARY_PATH="./target/release/$BINARY_NAME"
INSTALL_DIR="/usr/local/bin"
DESKTOP_ENTRY_DIR="/usr/share/applications"
ICON_DEST_DIR="/usr/share/icons/hicolor/256x256/apps"

echo "Installing $BINARY_NAME..."

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH."
    exit 1
fi

sudo cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

if [ -f "com.marcel.system-logs-diagnostic.png" ]; then
    sudo mkdir -p "$ICON_DEST_DIR"
    sudo cp com.marcel.system-logs-diagnostic.png "$ICON_DEST_DIR/com.marcel.system-logs-diagnostic.png"
    sudo gtk-update-icon-cache /usr/share/icons/hicolor/
fi

cat <<EOF | sudo tee "$DESKTOP_ENTRY_DIR/$APP_ID.desktop" > /dev/null
[Desktop Entry]
Name=System Diagnostic
Comment=Analyze system logs and kernel panics
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=com.marcel.system-logs-diagnostic.png
Terminal=false
Type=Application
Categories=System;Monitor;GTK;
StartupWMClass=$BINARY_NAME
Keywords=log;diagnostic;kernel;panic;
EOF

sudo update-desktop-database "$DESKTOP_ENTRY_DIR"

echo "Installation complete."
