#!/bin/bash

version=$(grep -m1 '^version' Cargo.toml | sed -E 's/version *= *"(.*)"/\1/')

echo "[Desktop Entry]"
echo "Name=bRAC"
echo "Version=$version"
echo "Type=Application"
echo "Comment=better RAC client"
echo "Icon=ru.themixray.bRAC"
echo "Exec=/usr/bin/bRAC"
echo "Categories=Network;"
echo "StartupNotify=true"
echo "Terminal=false"
echo "X-GNOME-UsesNotifications=true"
