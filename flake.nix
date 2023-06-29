{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla/master";
  };

  outputs = { nixpkgs, nixpkgs-mozilla, ... }:
    let
      inherit (nixpkgs) lib;

      makePackage = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ nixpkgs-mozilla.overlays.rust ];
          };

          rustPlatform =
            let
              rust = (pkgs.rustChannelOf {
                channel = "1.70.0";
                sha256 = "sha256-gdYqng0y9iHYzYPAdkC/ka3DRny3La/S5G8ASj0Ayyc=";
              }).rust.override {
                extensions = if dev then [ "rust-src" ] else [ ];
              };
            in
            pkgs.makeRustPlatform {
              cargo = rust;
              rustc = rust;
            };
        in
        rec {
          default = rustPlatform.buildRustPackage {
            name = "chatsounds-cli";
            src = lib.cleanSourceWith rec {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  let
                    baseName = builtins.baseNameOf (builtins.toString path);
                    relPath = lib.removePrefix (builtins.toString ./.) (builtins.toString path);
                  in
                  lib.any (re: builtins.match re relPath != null) [
                    "/\.cargo"
                    "/\.cargo/.*"
                    "/build.rs"
                    "/Cargo.lock"
                    "/Cargo.toml"
                    "/src"
                    "/src/.*"
                  ]
                );
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "chatsounds-0.2.0" = "sha256-CWUVl2aCjYGU8UUIeq8jFy/x+0/sQwBlKxF1mmUuCkQ=";
              };
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              # rustPlatform.bindgenHook
            ];

            buildInputs = with pkgs; [
              openssl
              alsa-lib
            ];

            # doCheck = false;
          };
        }
      );
    in
    builtins.foldl' lib.recursiveUpdate { } (builtins.map
      (system: {
        devShells.${system} = makePackage system true;
        packages.${system} = makePackage system false;
      })
      lib.systems.flakeExposed);
}
