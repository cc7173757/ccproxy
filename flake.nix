{
  description = "A reverse proxy for Minecraft Bedrock Edition servers.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    crane = {
      url = "github:ipetkov/crane";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      advisory-db,
      nix2container,
    }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      nix2containerPkgs = nix2container.packages.${system};
      craneLib = crane.mkLib pkgs;

      src = craneLib.cleanCargoSource ./.;

      commonArgs = {
        inherit src;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      crate = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
        }
      );
    in
    {
      checks.${system} = {
        inherit crate;

        crate-clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );

        crate-fmt = craneLib.cargoFmt {
          inherit src;
        };

        crate-audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        crate-deny = craneLib.cargoDeny {
          inherit src;
        };
      };

      packages.${system} = {
        default = crate;

        # Build a container image.
        container = nix2containerPkgs.nix2container.buildImage {
          name = "ccproxy";
          tag = "latest";
          copyToRoot = [ crate ];
          config = {
            entrypoint = [ "/bin/ccproxy" ];
            cmd = [ "run" ];
          };
        };
      };
    };
}
