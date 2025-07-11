#!/bin/bash

SUPPORTED_SYSTEMS=('')

build() {
    local build_dir=build/$1

    rm -rf $build_dir
    mkdir $build_dir
    mkdir $build_dir/misc

    $1 # build
    
    # copy readme, license and make a zip
    cp README.md $build_dir
    cp LICENSE $build_dir    
    zip -r $build_dir.zip $build_dir
}

SUPPORTED_SYSTEMS+=('linux-x86_64')
linux-x86_64() {
    # add gnotification version and install script
    cargo build -r
    cp target/release/bRAC $build_dir/misc/bRAC-gnotif
    cp misc/bRAC.png $build_dir/misc
    
    cp misc/user-install.sh $build_dir/install.sh
    cp misc/user-uninstall.sh $build_dir/uninstall.sh
    cp misc/create-desktop.sh $build_dir/misc

    chmod +x $build_dir/install.sh
    chmod +x $build_dir/uninstall.sh
    chmod +x $build_dir/misc/create-desktop.sh

    # add libnotify version as the alternative
    cargo build -r -F libnotify
    cp target/release/bRAC build/linux-x86_64
}

SUPPORTED_SYSTEMS+=('windows-x86_64')
windows-x86_64() {
    docker run -ti -v `pwd`:/mnt mglolenstine/gtk4-cross:rust-gtk-nightly /bin/bash -c "
    source \"\$HOME/.cargo/env\"; 
    rustup update nightly;                             # update nightly toolchain
    rustup +nightly target add x86_64-pc-windows-gnu;  # install windows stuff
    sed -i -e 's/cargo build/cargo +nightly build -F notify-rust,winapi/g' /bin/build;  # add features + nightly
    build;    # build it, creates package dir
    package;  # package it (adds some libs)
    mv package $build_dir;
    chmod -R 777 $build_dir;
    chmod -R 777 target"
}            

mkdir -p build

if [ $# -eq 0 ]; then
    for system in "${SUPPORTED_SYSTEMS[@]}"; do
        if [ ! -d build/$system ]; then build $system; fi
    done
else
    if [ $1 = "clean" ]; then
        rm -rf build
    elif [[ ${SUPPORTED_SYSTEMS[@]} =~ " $1" ]]; then
        build $1;
    fi
fi

