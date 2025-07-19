# How to compile it

## Windows

1. Install [rustup](https://rustup.rs/)
2. Install [MSVC](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and run `rustup default stable-msvc`
3. Extract [GTK4 from gvsbuild](https://github.com/wingtk/gvsbuild/releases/latest) to `C:\gtk` 
4. Update environment variables:
    - Go to Start, search for 'Advanced system settings' (or click on Properties of My Computer in the Explorer, then you'll find 'Advanced system settings')
    - Click 'Environment Variables...'
    - Add `C:\gtk\lib\pkgconfig` to the PKG_CONFIG_PATH variable (or create one if doesnt exist)
    - Add `C:\gtk\bin` to the PATH variable (or create one if doesnt exist)
    - Add `C:\gtk\lib` to the Lib variable (or create one if doesnt exist)
    - Apply and close the window (maybe restart PC)
5. Open the repository directory in console (download it from github or with `git clone https://github.com/MeexReay/bRAC.git`)
6. Run `cargo build -r -F winapi,notify-rust`
7. Done! Your finished binary is in the `target/release` folder.

## Linux / MacOS

1. Install `rust`, `openssl-dev`, `gtk4-dev` with your package manager
2. Open the repository directory in console (download it from github or with `git clone https://github.com/MeexReay/bRAC.git`)
3. Run `cargo build -r`
4. Done! Your finished binary is in the `target/release` folder.

# Troubleshooting

## Windows

### Black frame around the window

Black frame appears on connecting to the server or when bRAC just freezes. Be patient.

## MacOS

### Notifications dont work

There are two solutions:

- Switch to `libnotify`:

Add the feature `libnotify` to cargo: `cargo build -r -F libnotify`

- Switch to `notify-rust`:

Add the feature `notify-rust` to cargo: `cargo build -r -F notify-rust`

## Linux

### Notifications dont work

There are two solutions:

- Switch to `libnotify`:

Just add the new feature to cargo: `cargo build -r -F libnotify` \
Libnotify sucks in many situations, but it always work

- Make a desktop file:

Enter the repository folder and run: `./misc/create-desktop.sh` \
You'll get a desktop file contents, just edit paths here and write it to a new file in the `~/.local/share/applications` or `/usr/share/applications`\
All of these, with adding icons and other, makes this command: `make install` (using `gnumake` package) \
But make sure, that you have `.local/bin` in the `PATH` variable, otherwise it won't work. \
Now, if you'll run with the desktop file, GNotifications will work perfectly.

### Error: have you installed the static version of the ? library

Use this rustflags:

```bash
RUSTFLAGS="-C target-feature=-crt-static" cargo build -r
```

### Dark theme dont work

Edit `~/.config/gtk-4.0/settings.ini`:

```ini
[Settings]
gtk-application-prefer-dark-theme=1
```

# Cross-compiling (from Linux)

Build for all supported systems:

```bash
./misc/build.sh
```

The result will be in `build/` directory.

Supported systems:

- `windows-x86_64`
- `linux-x86_64`

To clean up the build, run `./misc/build.sh clean`

To build for only one system, run `./misc/build.sh <system>`,
example: `./misc/build.sh linux-x86_64`
