#!/bin/bash

version=$(grep -m1 '^version' Cargo.toml | sed -E 's/version *= *"(.*)"/\1/')

echo "[Desktop Entry]"
echo "Name=bRAC"
echo "Version=$version"
echo "Type=Application"
echo "Comment=better RAC client"
echo "Icon=$HOME/.local/share/bRAC/icon.png"
echo "Exec=$HOME/.local/bin/bRAC"
echo "Categories=Network;"
echo "StartupNotify=true"
echo "DBusActivatable=true"
echo "Terminal=false"
echo "X-GNOME-UsesNotifications=true"
