#!/bin/bash

mkdir -p /usr/bin
mkdir -p /usr/share
mkdir -p /usr/share/pixmaps

cp misc/bRAC-gnotif /usr/bin/bRAC
chmod +x /usr/bin/bRAC

cp misc/bRAC.png /usr/share/pixmaps/ru.themixray.bRAC.png
chmod +x misc/create-desktop.sh
./misc/create-desktop.sh > /usr/share/applications/ru.themixray.bRAC.desktop
