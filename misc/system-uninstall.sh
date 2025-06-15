#!/bin/bash

echo "this script is deprecated, fix it yourself if you wanna to"; exit

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi

getent passwd | while IFS=: read -r name password uid gid gecos home shell; do
    rm -rf $home/.config/bRAC;
done

rm -f /bin/bRAC
rm -f /usr/share/pixmaps/ru.themixray.bRAC.png
rm -f /usr/share/applications/ru.themixray.bRAC.desktop
