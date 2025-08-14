{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      rustManifest = lib.importTOML ./Cargo.toml;

      revSuffix = lib.optionalString (self ? shortRev || self ? dirtyShortRev)
        "-${self.shortRev or self.dirtyShortRev}";

      makePackages = (pkgs: rec {
        default = pkgs.rustPlatform.buildRustPackage rec {
          pname = rustManifest.package.name;
          version = rustManifest.package.version + revSuffix;

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

          preCheck =
            let
              cache-path = (fetched:
                builtins.concatStringsSep "/" (
                  [ "chatsounds" ] ++
                  builtins.match "(..)(.+)" (
                    builtins.hashString "sha256" (
                      builtins.replaceStrings [ "%20" ] [ " " ] fetched.url
                    )
                  )
                )
              );
            in
            ''
              mkdir -vp \
                "$(dirname ${cache-path hl1-msgpack})" \
                "$(dirname ${cache-path hl1-test-sound})"
              cp -v ${hl1-msgpack} "${cache-path hl1-msgpack}"
              cp -v ${hl1-test-sound} "${cache-path hl1-test-sound}"
              stat ${cache-path hl1-msgpack}
              stat ${cache-path hl1-test-sound}
            '';

          meta.mainProgram = pname;
        };

        hl1-msgpack = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/PAC3-Server/chatsounds-valve-games/HEAD/hl1/list.msgpack";
          hash = "sha256-ArdqCFv0wjiElqp6cwRZA/iFuaALr2silJh3STBgCl8=";
        };
        hl1-test-sound = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/PAC3-Server/chatsounds-valve-games/HEAD/hl1/scientist/ah%20hello%20gordon%20freeman%20its%20good%20to%20see%20you.ogg";
          hash = "sha256-QHHEXW18p2dtH/4ph0XGLZ5uvVokBQf+Njce37QWXnc=";
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
