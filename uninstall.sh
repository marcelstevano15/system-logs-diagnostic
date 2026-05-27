#!/usr/bin/env bash
set -e

APP_ID="com.marcel.system-logs-diagnostic"
BINARY_NAME="system-logs-diagnostic"

echo "Uninstalling $APP_ID..."

# Remove binary
sudo rm -f /usr/local/bin/$BINARY_NAME

# Remove desktop entry
sudo rm -f /usr/share/applications/$APP_ID.desktop

# Remove root/default icon
sudo rm -f /usr/share/pixmaps/$APP_ID.png

# Remove hicolor icons
for size in 48x48 64x64 96x96 128x128 256x256 512x512; do
    sudo rm -f \
        /usr/share/icons/hicolor/$size/apps/$APP_ID.png
done

# Remove LICENSE
sudo rm -rf /usr/share/licenses/$BINARY_NAME

# Update icon cache
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true

# Update desktop database
sudo update-desktop-database /usr/share/applications || true

echo "Uninstalled successfully."
