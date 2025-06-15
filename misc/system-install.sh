#!/bin/bash

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi

cp bRAC /bin/bRAC
chmod +x /bin/bRAC
cp ru.themixray.bRAC.png /usr/share/pixmaps
cp ru.themixray.bRAC.desktop /usr/share/applications