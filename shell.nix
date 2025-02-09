{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    gcc_multi
    pkg-config
    zlib
    openssl
    pkgsCross.gnu32.buildPackages.gcc
    pkgsCross.mingw32.buildPackages.gcc
    pkgsCross.mingwW64.buildPackages.gcc
  ];
}