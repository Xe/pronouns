{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    Xess = {
      url = "github:Xe/Xess";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.utils.follows = "utils";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, utils, naersk, nixpkgs, Xess }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) { inherit system; };

        naersk' = pkgs.callPackage naersk { };

        xess = Xess.packages.${system}.default;
      in rec {
        # For `nix build` & `nix run`:
        packages = rec {
          bin = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = with pkgs; [ pkg-config openssl ];
            XESS_PATH = "${xess}/static/css";
          };

          data = pkgs.stdenv.mkDerivation {
            pname = "pronouns-data";
            inherit (bin) version;
            src = ./dhall;
            nativeBuildInputs = with pkgs; [ dhall ];

            phases = "installPhase";

            installPhase = ''
              mkdir -p $out/dhall
              dhall resolve --file $src/package.dhall >> $out/dhall/package.dhall
            '';
          };

          default = pkgs.symlinkJoin {
            name = "pronouns-${bin.version}";
            paths = [ bin data ];
          };

          docker = pkgs.dockerTools.buildLayeredImage {
            name = "registry.fly.io/xe-pronouns";
            tag = "latest";
            contents = [ default ];
            config = {
              Cmd = [ "${bin}/bin/pronouns" ];
              WorkingDir = default;
            };
          };
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            rustfmt
            python3
            pkg-config
            openssl
            dhall
            flyctl
            terraform
          ];

          XESS_PATH = "${xess}/static/css";
        };
      });
}
