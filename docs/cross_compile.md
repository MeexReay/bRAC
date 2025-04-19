# Cross-compile on Linux to Windows

## Install dev packages

on Nix:

```bash
nix-shell -p pkgsCross.mingwW64.stdenv.cc pkgsCross.mingwW64.windows.pthreads pkgsCross.mingwW64.gtk4
```

## Build

```bash
build build/windows-x86_64
```