#!/usr/bin/env bash
set -e

APP_NAME="system-logs-diagnostic"

sudo rm -f /usr/local/bin/$APP_NAME
sudo rm -f /usr/share/applications/$APP_NAME.desktop
sudo rm -f /usr/share/icons/hicolor/*/apps/$APP_NAME.*

sudo gtk-update-icon-cache /usr/share/icons/hicolor/ || true

echo "Uninstalled."
