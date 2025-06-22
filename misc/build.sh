#!/bin/bash

echo "Run this script only from repository root!"
echo "This script depends on:"
echo " - fact that you are on linux x86_64!"
echo " - wine. install it with your distro's package manager"
echo " - cross crate. to install it, run this: cargo install cross --git https://github.com/cross-rs/cross"
echo " - docker, so you should run something like this on your distro: sudo systemctl start docker"
read -p "Press enter if you really want to do rm -rf build/"

rm -rf build
mkdir build

build_linux() {
    mkdir build/linux-x86_64
    mkdir build/linux-x86_64/misc
    cargo build -r
    cp target/release/bRAC build/linux-x86_64/misc/bRAC-gnotif
    cp misc/user-install.sh build/linux-x86_64
    cp misc/bRAC.png build/linux-x86_64/misc
    cp misc/create-desktop.sh build/linux-x86_64/misc
    cargo build -r -F libnotify
    cp target/release/bRAC build/linux-x86_64
}

build_windows() {
    curl https://www.mamachine.org/mslink/mslink_v1.3.sh -o misc/mslink.sh
    chmod +x misc/mslink.sh
    curl https://github.com/wingtk/gvsbuild/releases/download/2025.5.0/GTK4_Gvsbuild_2025.5.0_x64.zip -o build/gvsbuild.zip
    unzip build/gvsbuild.zip "bin/*" -d build/windows-x86_64
    rm build/gvsbuild.zip
    cross build --target x86_64-pc-windows-gnu -r
    cp target/release/bRAC.exe build/windows-x86_64/bin
    ./misc/mslink.sh -l bin\\bRAC.exe -o build/windows-x86_64/bRAC.lnk
}

build_linux
build_windows