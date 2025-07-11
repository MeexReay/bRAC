#!/bin/bash

mkdir -p ~/.local
mkdir -p ~/.local/bin
mkdir -p ~/.local/share
mkdir -p ~/.local/share/bRAC

cp misc/bRAC-gnotif ~/.local/bin/bRAC
chmod +x ~/.local/bin/bRAC

cp misc/bRAC.png ~/.local/share/bRAC/icon.png
chmod +x misc/create-desktop.sh
./misc/create-desktop.sh > ~/.local/share/applications/ru.themixray.bRAC.desktop
