{
  description =
    "Terminal world weather dashboard";

  inputs = {
    nixpkgs.url =
      "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url =
      "github:numtide/flake-utils";

    rust-overlay = {
      url =
        "github:oxalica/rust-overlay";

      inputs.nixpkgs.follows =
        "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:

    flake-utils.lib.eachDefaultSystem (
      system:

      let
        pkgs =
          import nixpkgs {
            inherit system;

            overlays = [
              (import rust-overlay)
            ];
          };

        rustToolchain =
          pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
              "rustfmt"
            ];
          };

        defaultPeople = [
          {
            name =
              "YOU";

            location =
              "Hyderabad";

            timezone =
              "Asia/Kolkata";
          }

          {
            name =
              "ALICE";

            location =
              "London";

            timezone =
              "Europe/London";
          }

          {
            name =
              "BOB";

            location =
              "New York";

            timezone =
              "America/New_York";
          }

          {
            name =
              "CHARLIE";

            location =
              "Tokyo";

            timezone =
              "Asia/Tokyo";
          }
        ];

        mkWorldDashboard =
          {
            people ? defaultPeople,
          }:

          pkgs.callPackage
            ./nix/package.nix
            {
              inherit
                rustToolchain
                people;
            };

        worldDashboard =
          mkWorldDashboard {};

      in
      {
        packages = {
          default =
            worldDashboard;

          world-dashboard =
            worldDashboard;
        };

        apps.default = {
          type =
            "app";

          program =
            "${worldDashboard}/bin/world-dashboard";
        };

        devShells.default =
          pkgs.mkShell {
            packages = [
              rustToolchain
              pkgs.pkg-config
            ];
          };

        lib = {
          inherit
            mkWorldDashboard;
        };
      }
    );
}
