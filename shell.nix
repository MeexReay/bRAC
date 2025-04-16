{ pkgs ? import <nixpkgs> {} }:
  let 
    devDeps = with pkgs; [ 
      pkg-config 
      openssl 
      gtk4 
      pango
    ];
  in pkgs.mkShell {
    shellHook = ''
      export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
    '';
    buildInputs = devDeps;
    nativeBuildInputs = with pkgs; [ 
      rustc
      cargo
    ] ++ devDeps;
  }