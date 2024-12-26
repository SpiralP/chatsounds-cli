{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      rustManifest = lib.importTOML ./Cargo.toml;

      makePackages = (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage rec {
          pname = rustManifest.package.name;
          version = "${rustManifest.package.version}-${self.shortRev or self.dirtyShortRev}";

          src = lib.sourceByRegex ./. [
            "^\.cargo(/.*)?$"
            "^build\.rs$"
            "^Cargo\.(lock|toml)$"
            "^src(/.*)?$"
          ];

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          buildInputs = with pkgs; [
            alsa-lib
            openssl
          ];

          nativeBuildInputs = with pkgs; [
            makeWrapper
            pkg-config
          ];

          meta.mainProgram = pname;
        };
      });
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
