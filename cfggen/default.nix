{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "balsa-configgen";
  version = "0.1.0";

  # Explicit fileset keeps target/ and editor state out of the build.
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./tests
      ./test-plans
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  # Needs podman and network, neither of which exists in the build sandbox.
  checkFlags = [ "--skip=every_fixture_generates_and_evaluates" ];

  meta.mainProgram = "balsa-configgen";
}
