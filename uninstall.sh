#!/bin/bash

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi

getent passwd | while IFS=: read -r name password uid gid gecos home shell; do
    rm -rf $home/.config/bRAC;
done

rm -f /bin/bRAC
rm -f ru.themixray.bRAC.png /usr/share/pixmaps
rm -f ru.themixray.bRAC.desktop /usr/share/applications