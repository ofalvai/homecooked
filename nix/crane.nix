{
  pkgs,
  lib,
  crane,
  toolchain,
  advisory-db,
}:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

  src = craneLib.cleanCargoSource (craneLib.path ../.);

  # Common arguments can be set here to avoid repeating them later
  commonArgs = {
    inherit src;
    strictDeps = true;

    buildInputs = [
      pkgs.libxml2
    ];
    nativeBuildInputs = [ pkgs.pkg-config ];
  };

  # Build *just* the cargo dependencies (of the entire workspace),
  # so we can reuse all of that work (e.g. via cachix) when running in CI
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{
  inherit craneLib cargoArtifacts;

  individualCrateArgs = commonArgs // {
    inherit cargoArtifacts;
    inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
    # NB: we disable tests since we'll run them all via cargo-nextest
    doCheck = false;
  };

  fileSetForCrate =
    crate:
    lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../Cargo.toml
        ../Cargo.lock
        (craneLib.fileset.commonCargoSources ../crates/homecooked-hack)
        (craneLib.fileset.commonCargoSources crate)
      ];
    };

  checks = import ./checks.nix {
    inherit
      pkgs
      craneLib
      src
      commonArgs
      cargoArtifacts
      advisory-db
      ;
  };

  packages = {
    workspaceAll = craneLib.cargoBuild (
      commonArgs
      // {
        inherit cargoArtifacts;
        doCheck = true;
      }
    );
  };
}
