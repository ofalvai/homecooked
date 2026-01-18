{
  description = "Nix configuration for the Homecooked monorepo";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      ...
    }@inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };
        inherit (pkgs) lib;

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        crane = import ./nix/crane.nix {
          inherit
            pkgs
            lib
            toolchain
            ;
          crane = inputs.crane;
          advisory-db = inputs.advisory-db;
        };

        embeddings = import ./crates/embeddings {
          inherit (crane) craneLib individualCrateArgs fileSetForCrate;
        };
        focus = import ./crates/focus {
          inherit (crane) craneLib individualCrateArgs fileSetForCrate;
        };
        gardener = import ./crates/gardener {
          inherit (crane) craneLib individualCrateArgs fileSetForCrate;
        };
        llm-assistant = import ./crates/llm-assistant {
          inherit lib;
          inherit (crane) craneLib individualCrateArgs;
        };
        llm-toolkit = import ./crates/llm-toolkit {
          inherit (crane) craneLib individualCrateArgs fileSetForCrate;
        };
        speedtest-to-influx = import ./crates/speedtest-to-influx {
          inherit (crane) craneLib individualCrateArgs fileSetForCrate;
        };

      in
      {
        # nix develop
        devShells.default = import ./nix/devshell.nix {
          inherit
            self
            pkgs
            system
            toolchain
            ;
          craneLib = crane.craneLib;
        };

        # nix build .#<name>
        packages = {
          inherit
            embeddings
            focus
            gardener
            llm-assistant
            llm-toolkit
            speedtest-to-influx
            ;

          deps = crane.cargoArtifacts;
          ci = crane.packages.workspaceAll;

          docker-llm-assistant = import ./crates/llm-assistant/docker.nix {
            inherit pkgs llm-assistant;
          };
        };

        # Executed by `nix flake check`
        # Hint: run individual checks with `nix build .#checks.<name>`
        checks = crane.checks // {
          # Build the crates as part of `nix flake check` for convenience
          inherit
            embeddings
            focus
            gardener
            llm-assistant
            llm-toolkit
            speedtest-to-influx
            ;
        };

        # nix run .#<name>
        apps = {
          embeddings = inputs.flake-utils.lib.mkApp { drv = embeddings; };
          llm-assistant = inputs.flake-utils.lib.mkApp { drv = llm-assistant; };
        };

        # nix fmt
        formatter = pkgs.nixfmt;
      }
    );
}
