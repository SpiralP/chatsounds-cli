{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      makePackage = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            name = "chatsounds-cli";
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  lib.any (re: builtins.match re (lib.removePrefix (builtins.toString ./.) (builtins.toString path)) != null) [
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
                "chatsounds-0.2.0" = "sha256-PnggDT0oWtRRowrGoD8Bi8+Fpss6SKzQ1PDk3n1tCBM=";
              };
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ] ++ (if dev then
              with pkgs; [
                clippy
                rustfmt
                rust-analyzer
              ] else [ ]);

            buildInputs = with pkgs; [
              openssl
              alsa-lib
            ];
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
