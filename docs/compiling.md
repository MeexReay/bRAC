# How to compile it

## Windows

1. Install [rustup](https://rustup.rs/)
2. Install [MSVC](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and run `rustup default stable-msvc`
3. Extract [GTK4 from gvsbuild](https://github.com/wingtk/gvsbuild/releases/tag/latest) to `C:\gtk` 
4. Update environment variables:
    - Go to Start
    - Search for 'Advanced system settings'
    - Click 'Environment Variables...'
    - Add `C:\gtk\lib\pkgconfig` to the PKG_CONFIG_PATH variable
    - Add `C:\gtk\bin` to the PATH variable
    - Add `C:\gtk\lib` to the Lib variable
    - Save and exit
5. Open the repository directory in console (download it from github or with `git clone https://github.com/MeexReay/bRAC.git`)
6. Run `cargo build -r -F libnotify,winapi`
7. Done! Your finished binary is in the `target/release` folder.

## Linux / MacOS

1. Install `rust`, `openssl-dev`, `gtk4-dev` with your package manager
2. Open the repository directory in console (download it from github or with `git clone https://github.com/MeexReay/bRAC.git`)
3. Run `cargo build -r`
4. Done! Your finished binary is in the `target/release` folder.

# Troubleshooting

## Windows / MacOS

### Doesnt compile / doesnt work

Write a new issue here and dont google anything!!1

## Linux

### Notifications dont work

There are Two solutions:

- Switch to `libnotify`:

Just add the new feature to cargo: `cargo build -r -F libnotify` \
Libnotify sucks in many situations, but it always work

- Make a desktop file:

Enter the repository folder and run: `./misc/create-desktop.sh` \
You'll get a desktop file contents, just edit paths here and write it to a new file in the `~/.local/share/applications` or `/usr/share/applications`\
All of these, with adding icons and other, makes this command: `make install` (using `gnumake` package) \
But make sure, that you have `.local/bin` in the `PATH` variable, otherwise it won't work. \
Now, if you'll run with the desktop file, GNotifications will work perfectly.
