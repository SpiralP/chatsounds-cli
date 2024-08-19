{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      makePackages = (pkgs:
        let
          rustManifest = lib.importTOML ./Cargo.toml;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = rustManifest.package.name;
            version = rustManifest.package.version;

            src = lib.sourceByRegex ./. [
              "^\.cargo(/.*)?$"
              "^build\.rs$"
              "^Cargo\.(lock|toml)$"
              "^src(/.*)?$"
            ];

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "chatsounds-0.2.0" = "sha256-YRGKgzwTmBMEjTcw3SLidrOFWmWAqqf0u3yV6aqxBns=";
              };
            };

            buildInputs = with pkgs; [
              alsa-lib
              openssl
            ];

            nativeBuildInputs = with pkgs; [
              makeWrapper
              pkg-config
            ];
          };
        }
      );
    in
    builtins.foldl' lib.recursiveUpdate { } (builtins.map
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          packages = makePackages pkgs;
        in
        {
          devShells.${system} = packages // {
            default =
              let
                allDrvsIn = (name:
                  lib.lists.flatten (
                    builtins.map
                      (drv: drv.${name} or [ ])
                      (builtins.attrValues packages)
                  ));
              in
              pkgs.mkShell {
                name = "dev-shell";
                packages = with pkgs; [
                  clippy
                  (rustfmt.override { asNightly = true; })
                  rust-analyzer
                ];
                buildInputs = allDrvsIn "buildInputs";
                nativeBuildInputs = allDrvsIn "nativeBuildInputs";
                propagatedBuildInputs = allDrvsIn "propagatedBuildInputs";
                propagatedNativeBuildInputs = allDrvsIn "propagatedNativeBuildInputs";
              };
          };
          packages.${system} = packages;
        })
      lib.systems.flakeExposed);
}
