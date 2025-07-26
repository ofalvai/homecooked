{
  description = "Build a cargo workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
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
      nixpkgs,
      crane,
      fenix, # TODO: use a specific minor Rust version from nixpkgs
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib nixpkgs.legacyPackages.${system};
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs =
            [ pkgs.libxml2 ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

          nativeBuildInputs = [ pkgs.pkg-config ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        craneLibLLvmTools = craneLib.overrideToolchain (
          fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]
        );

        individualCrateArgs = commonArgs // {
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./homecooked-hack
              # ./. # TODO: this hurts cacheability, but is necessary for now
              crate
            ];
          };

        # Build *just* the cargo dependencies (of the entire workspace),
        # so we can reuse all of that work (e.g. via cachix) when running in CI
        # It is *highly* recommended to use something like cargo-hakari to avoid
        # cache misses when building individual top-level-crates
        workspaceDeps = craneLib.buildDepsOnly commonArgs;

        workspaceAll = craneLib.cargoBuild (
          commonArgs
          // {
            cargoArtifacts = workspaceDeps;
            doCheck = true;
          }
        );

        embeddings = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "embeddings";
            cargoExtraArgs = "-p embeddings";
            src = fileSetForCrate ./embeddings;
          }
        );
        focus = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "focus";
            cargoExtraArgs = "-p focus";
            src = fileSetForCrate ./focus;
          }
        );
        gardener = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "gardener";
            cargoExtraArgs = "-p gardener";
            src = fileSetForCrate ./gardener;
          }
        );
        llm-assistant = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "llm-assistant";
            cargoExtraArgs = "-p llm-assistant";
            src = fileSetForCrate ./llm-assistant;
          }
        );
        llm-toolkit = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "llm-toolkit";
            cargoExtraArgs = "-p llm-toolkit";
            src = fileSetForCrate ./llm-toolkit;
          }
        );
        speedtest-to-influx = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "speedtest-to-influx";
            cargoExtraArgs = "-p speedtest-to-influx";
            src = fileSetForCrate ./speedtest-to-influx;
          }
        );
      in
      {
        # Executed by `nix flake check`
        # Hint: run individual checks with `nix build .#checks.<name>`
        checks = {
          # Build the crates as part of `nix flake check` for convenience
          # inherit embeddings focus gardener llm-assistant llm-toolkit speedtest-to-influx;

          # Run clippy (and deny all warnings) on the workspace source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.

          # TODO: enable once lint errors are fixed
          # clippy = craneLib.cargoClippy (commonArgs
          #   // {
          #     cargoArtifacts = workspaceDeps;
          #     cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          #   });

          # Check formatting
          fmt = craneLib.cargoFmt { inherit src; };

          # Run tests with cargo-nextest
          test = craneLib.cargoNextest (
            commonArgs
            // {
              cargoArtifacts = workspaceDeps;
              partitions = 1;
              partitionType = "count";
            }
          );

          # Ensure that cargo-hakari is up to date
          hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "homecooked-hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
              cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
              cargo hakari verify
            '';

            nativeBuildInputs = [ pkgs.cargo-hakari ];
          };
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
          deps = workspaceDeps;
          ci = workspaceAll;

          docker = {
            llm-assistant = pkgs.dockerTools.buildLayeredImage {
              name = "ghcr.io/ofalvai/homecooked-llm-assistant";
              tag = "latest";
              config = {
                WorkingDir = "/app";
                Cmd = [
                  "${llm-assistant}/bin/llm-assistant"
                  "--config"
                  "/data/config/config.ini"
                  "server"
                ];
                ExposedPorts = {
                  "8080/tcp" = { };
                };
                Volumes = {
                  "/data/config" = { };
                };
                Env = [
                  "CONFIG=/data/config/config.ini"
                  "PORT=8080"
                ];
              };
            };
          };
        };

        # nix run .#<name>
        apps = {
          embeddings = flake-utils.lib.mkApp { drv = embeddings; };
          llm-assistant = flake-utils.lib.mkApp { drv = llm-assistant; };
        };

        # nix develop
        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [ pkgs.cargo-hakari ];
        };

        # nix fmt
        formatter = flake-utils.lib.eachDefaultSystem (system: nixpkgs.legacyPackages.${system}.nixfmt-rfc-style);
      }
    );
}
