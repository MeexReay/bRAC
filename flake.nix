{
  description = "bRAC - better RAC client";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          devDeps = with pkgs; [ pkg-config openssl gtk4 pango ];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          msrv = cargoToml.package.rust-version;

          mkDevShell = rustc:
            pkgs.mkShell {
              shellHook = ''
                export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
              '';
              buildInputs = devDeps;
              nativeBuildInputs = devDeps ++ [ rustc ];
            };
        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          packages.default = self'.packages.bRAC;
          devShells.default = self'.devShells.stable;

          packages.bRAC = let 
              deps = [
                pkg-config
                openssl
                gtk4
                pango
              ];
            in (pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            }).buildRustPackage {
              inherit (cargoToml.package) name;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              version = lib.concatStrings [ cargoToml.package.version version ];
              buildInputs = deps;
              nativeBuildInputs = deps;
              patchPhase = ''
                substituteInPlace Cargo.toml --replace \
                  'version = "${cargoToml.package.version}"' \
                  'version = "${lib.concatStrings [ cargoToml.package.version version ]}"'
              '';
            };

          devShells.nightly = (mkDevShell (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)));
          devShells.stable = (mkDevShell pkgs.rust-bin.stable.latest.default);
          devShells.msrv = (mkDevShell pkgs.rust-bin.stable.${msrv}.default);
        };
    };
}