#!/bin/bash

# echo "Run this script only from repository root!"
# echo "This script depends on:"
# echo " - fact that you are on linux x86_64!"
# echo " - zip, unzip, curl. install it with your distro's package manager"
# echo " - cross crate. to install it, run this: cargo install cross --git https://github.com/cross-rs/cross"
# echo " - docker, so you should run something like this on your distro: sudo systemctl start docker"
# read -p "Press enter if you really want to do rm -rf build/"

build_linux() {
    mkdir build/linux-x86_64
    mkdir build/linux-x86_64/misc

    # add gnotification version and install script
    cargo build -r
    cp target/release/bRAC build/linux-x86_64/misc/bRAC-gnotif
    cp misc/user-install.sh build/linux-x86_64/install.sh
    cp misc/bRAC.png build/linux-x86_64/misc
    cp misc/create-desktop.sh build/linux-x86_64/misc

    # add libnotify version as the alternative
    cargo build -r -F libnotify
    cp target/release/bRAC build/linux-x86_64
    
    # copy readme, license and make a zip
    cp README.md build/linux-x86_64
    cp LICENSE build/linux-x86_64    
    zip -r build/bRAC-linux-x86_64.zip build/linux-x86_64
}

build_windows() {
    docker run -ti -v `pwd`:/mnt mglolenstine/gtk4-cross:rust-gtk-nightly /bin/bash -c "
    source \"\$HOME/.cargo/env\"; 
    rustup update nightly;                             # update nightly toolchain
    rustup +nightly target add x86_64-pc-windows-gnu;  # install windows stuff
    sed -i -e 's/cargo build/cargo +nightly build -F notify-rust,winapi/g' /bin/build;  # add features + nightly
    build;    # build it, creates package dir
    package;  # package it (adds some libs)
    mv package build/windows-x86_64;
    chmod -R 777 build/windows-x86_64;
    chmod -R 777 target"
    
    # copy readme, license and make a zip
    cp README.md build/windows-x86_64
    cp LICENSE build/windows-x86_64
    zip -r build/bRAC-windows-x86_64.zip build/windows-x86_64
}                                      

mkdir -p build

if [ $# -eq 0 ]; then
    if [ ! -d build/windows-x86_64 ]; then
        build_windows
    fi
    if [ ! -d build/linux-x86_64 ]; then
        build_linux
    fi
    exit
fi

if [ $1 = "clean" ]; then
    rm -rf build
elif [ $1 = "windows" ]; then
    rm -rf build/windows-x86_64
    build_windows
elif [ $1 = "linux" ]; then
    rm -rf build/linux-x86_64
    build_linux
else
  echo "possible arguments: clean windows linux. none for auto"
fi
