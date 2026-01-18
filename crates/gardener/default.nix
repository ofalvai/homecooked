{
  craneLib,
  individualCrateArgs,
  fileSetForCrate,
}:
craneLib.buildPackage (
  individualCrateArgs
  // {
    pname = "gardener";
    cargoExtraArgs = "-p gardener";
    src = fileSetForCrate ./.;
  }
)
