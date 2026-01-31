{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:SpiralP/nix-flake-utils";
  };

  outputs = inputs@{ flake-utils, ... }:
    flake-utils.lib.makeOutputs inputs
      ({ lib, pkgs, makeRustPackage, dev, ... }: rec {
        hl1-msgpack = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/PAC3-Server/chatsounds-valve-games/HEAD/hl1/list.msgpack";
          hash = "sha256-ArdqCFv0wjiElqp6cwRZA/iFuaALr2silJh3STBgCl8=";
        };
        hl1-test-sound = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/PAC3-Server/chatsounds-valve-games/HEAD/hl1/scientist/ah%20hello%20gordon%20freeman%20its%20good%20to%20see%20you.ogg";
          hash = "sha256-QHHEXW18p2dtH/4ph0XGLZ5uvVokBQf+Njce37QWXnc=";
        };

        default = makeRustPackage pkgs (self: {
          src = ./.;

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
        });
      });
}
