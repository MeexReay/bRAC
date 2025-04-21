#!/bin/bash

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi

if [ ! -f bRAC ]; then
    if cargo build -r; then
        cp target/release/bRAC .
    else
        echo "There is no bRAC binary"
        exit 1
    fi
fi

cp bRAC /bin/bRAC
chmod +x /bin/bRAC
cp ru.themixray.bRAC.png /usr/share/pixmaps
cp ru.themixray.bRAC.desktop /usr/share/applications