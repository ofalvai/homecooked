{
  craneLib,
  individualCrateArgs,
  fileSetForCrate,
}:
craneLib.buildPackage (
  individualCrateArgs
  // {
    pname = "focus";
    cargoExtraArgs = "-p focus";
    src = fileSetForCrate ./.;
  }
)
