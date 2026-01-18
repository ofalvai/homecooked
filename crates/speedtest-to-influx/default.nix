{
  craneLib,
  individualCrateArgs,
  fileSetForCrate,
}:
craneLib.buildPackage (
  individualCrateArgs
  // {
    pname = "speedtest-to-influx";
    cargoExtraArgs = "-p speedtest-to-influx";
    src = fileSetForCrate ./.;
  }
)
