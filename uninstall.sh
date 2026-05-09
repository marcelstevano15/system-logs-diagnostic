#!/usr/bin/env bash
set -e

APP_ID="com.marcel.system-logs-diagnostic"
BINARY_NAME="system-logs-diagnostic"

echo "Uninstalling $APP_ID..."

sudo rm -f /usr/local/bin/$BINARY_NAME
sudo rm -f /usr/share/applications/$APP_ID.desktop
sudo rm -f /usr/share/icons/hicolor/256x256/apps/$APP_ID.png
sudo rm -f /usr/share/icons/hicolor/scalable/apps/$APP_ID.svg || true

sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor/
sudo update-desktop-database /usr/share/applications

echo "Uninstalled successfully."

