{
  self,
  pkgs,
  system,
  craneLib,
  toolchain,
}:
craneLib.devShell {
  # Inherit inputs from checks.
  checks = self.checks.${system};

  # rust-analyzer needs the rust-src component from the toolchain we defined above
  RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";

  # Extra inputs can be added here; cargo and rustc are provided by default.
  packages = [
    pkgs.cargo-hakari
    pkgs.cargo-outdated
    pkgs.cargo-edit
  ];
}
