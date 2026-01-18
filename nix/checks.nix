{
  pkgs,
  advisory-db,
  src,
  craneLib,
  commonArgs,
  cargoArtifacts,
}:
{
  # Run clippy (and deny all warnings) on the workspace source,
  # again, reusing the dependency artifacts from above.
  #
  # Note that this is done as a separate derivation so that
  # we can block the CI if there are issues here, but not
  # prevent downstream consumers from building our crate by itself.
  clippy = craneLib.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    }
  );

  toml-fmt = craneLib.taploFmt {
    src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
  };

  # Check formatting
  fmt = craneLib.cargoFmt { inherit src; };

  # Run tests with cargo-nextest
  test = craneLib.cargoNextest (
    commonArgs
    // {
      inherit cargoArtifacts;
      partitions = 1;
      partitionType = "count";
    }
  );

  audit = craneLib.cargoAudit {
    inherit src advisory-db;
  };

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
}
