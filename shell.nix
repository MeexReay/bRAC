{ pkgs ? import <nixpkgs> {} }:

let
  # Переопределение для mingw32
  mingw32WithDwarf = pkgs.pkgsCross.mingw32.buildPackages.gcc.overrideAttrs (oldAttrs: {
    configureFlags = [
      "--disable-sjlj-exceptions"
      "--enable-dwarf2"
    ];
  });
in

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    gcc_multi
    pkg-config
    zlib
    openssl

    # Добавляем кросс-компиляторы
    pkgsCross.gnu32.buildPackages.gcc
    pkgsCross.gnu32.buildPackages.binutils
    pkgsCross.gnu64.buildPackages.gcc
    pkgsCross.gnu64.buildPackages.binutils

    # Переопределённый MinGW для 32-бит Windows
    mingw32WithDwarf

    # Необходимые библиотеки для Windows
    pkgsCross.mingw32.windows.pthreads
    pkgsCross.mingw32.windows.mcfgthreads

    # 64-битный MinGW и необходимые библиотеки
    pkgsCross.mingwW64.buildPackages.gcc
    pkgsCross.mingwW64.windows.pthreads
    pkgsCross.mingwW64.windows.mcfgthreads
  ];
}