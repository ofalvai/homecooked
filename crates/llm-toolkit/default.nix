{
  craneLib,
  individualCrateArgs,
  fileSetForCrate,
}:
craneLib.buildPackage (
  individualCrateArgs
  // {
    pname = "llm-toolkit";
    cargoExtraArgs = "-p llm-toolkit";
    src = fileSetForCrate ./.;
  }
)
