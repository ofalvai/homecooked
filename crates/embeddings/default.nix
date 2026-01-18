{
  craneLib,
  individualCrateArgs,
  fileSetForCrate,
}:
craneLib.buildPackage (
  individualCrateArgs
  // {
    pname = "embeddings";
    cargoExtraArgs = "-p embeddings";
    src = fileSetForCrate ./.;
  }
)
