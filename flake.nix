{
  description = "Cargo project with cross-compilation support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "x86_64-darwin"
      "x86_64-windows"
      "i686-linux"
      "i686-windows"
      "aarch64-darwin"
    ] (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          crossSystem = (import <nixpkgs/lib>).systems.examples.gnu64 // {
            rust.rustcTarget = {
              "x86_64-linux" = "x86_64-unknown-linux-gnu";
              "x86_64-darwin" = "x86_64-apple-darwin";
              "x86_64-windows" = "x86_64-pc-windows-gnu";
              "i686-linux" = "i686-unknown-linux-gnu";
              "i686-windows" = "i686-pc-windows-gnu";
              "aarch64-darwin" = "aarch64-apple-darwin";
            }.${system}; # here invalid target format
          };
        };
        
        exeSuffix = if pkgs.stdenv.hostPlatform.isWindows then ".exe" else "";
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            pkg-config
            (if stdenv.isDarwin then darwin.libiconv else null)
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "my-cargo-project";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          meta = with pkgs.lib; {
            description = "My Rust project";
            license = licenses.mit;
            maintainers = with maintainers; [ ];
            platforms = platforms.all;
          };

          postInstall = ''
            mv $out/bin/bRAC $out/bin/bRAC-${system}${exeSuffix}
          '';
        };
      });
}