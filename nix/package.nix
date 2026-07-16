{
  lib,
  rustPlatform,
  rustToolchain,
  makeWrapper,
  people,
}:

let
  peopleJson =
    builtins.toFile
      "people.json"
      (builtins.toJSON people);

in
rustPlatform.buildRustPackage.override
  {
    rustc = rustToolchain;
    cargo = rustToolchain;
  }
  {
    pname = "world-dashboard";

    version = "0.1.0";

    src = lib.cleanSource ../.;

    cargoLock = {
      lockFile = ../Cargo.lock;
    };

    nativeBuildInputs = [
      makeWrapper
    ];

    postInstall = ''
      mkdir -p $out/share/world-dashboard

      cp ${peopleJson} \
        $out/share/world-dashboard/people.json
    '';

    postFixup = ''
      wrapProgram $out/bin/world-dashboard \
        --set WORLD_DASHBOARD_CONFIG \
        "$out/share/world-dashboard/people.json"
    '';

    meta = {
      description =
        "Terminal world weather dashboard";

      license =
        lib.licenses.mit;
    };
  }
