{
  lib,
  craneLib,
  individualCrateArgs,
}:
craneLib.buildPackage (
  individualCrateArgs
  // {
    pname = "llm-assistant";
    cargoExtraArgs = "-p llm-assistant";
    src = lib.fileset.toSource {
      root = ../../.;
      fileset = lib.fileset.unions [
        ../../Cargo.toml
        ../../Cargo.lock
        (craneLib.fileset.commonCargoSources ../homecooked-hack)
        (craneLib.fileset.commonCargoSources ./.)
        (craneLib.fileset.commonCargoSources ../llm-toolkit)
      ];
    };
  }
)
