#!/bin/bash

cp bRAC ~/.local/bin/bRAC
chmod +x ~/.local/bin/bRAC
mkdir ~/.local/share/bRAC -p
cp misc/bRAC.png ~/.local/share/bRAC/icon.png
./misc/create-desktop.sh > ~/.local/share/applications/ru.themixray.bRAC.desktop
