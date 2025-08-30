{
  description = "bRAC - better RAC client";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        devDeps = with pkgs; [ pkg-config openssl gtk4 pango libnotify ];
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        mkDevShell = rustc:
          pkgs.mkShell {
            shellHook = ''
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            '';
            buildInputs = devDeps;
            nativeBuildInputs = devDeps ++ [ rustc ];
          };
      in {
        devShells.nightly = (mkDevShell (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)));
        devShells.default = (mkDevShell pkgs.rust-bin.stable.latest.default);

        packages.default = (pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.nightly.latest.minimal;
            rustc = pkgs.rust-bin.nightly.latest.minimal;
          }).buildRustPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildInputs = devDeps;
            nativeBuildInputs = devDeps;
          };
      }
    );
}
