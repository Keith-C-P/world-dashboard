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

    let
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
          pkgs,
          people ? defaultPeople,
        }:

        let
          rustToolchain =
            pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
                "clippy"
                "rustfmt"
              ];
            };

        in
        pkgs.callPackage
          ./nix/package.nix
          {
            inherit
              rustToolchain
              people;
          };

    in
    {
      lib =
        {
          inherit
            mkWorldDashboard;
        };

      packages =
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

            worldDashboard =
              mkWorldDashboard {
                inherit pkgs;
              };

          in
          {
            default =
              worldDashboard;

            world-dashboard =
              worldDashboard;
          }
        );

      apps =
        flake-utils.lib.eachDefaultSystem (
          system:

          let
            worldDashboard =
              self.packages.${system}.default;

          in
          {
            default = {
              type =
                "app";

              program =
                "${worldDashboard}/bin/world-dashboard";
            };
          }
        );

      devShells =
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

          in
          {
            default =
              pkgs.mkShell {
                packages = [
                  rustToolchain
                  pkgs.pkg-config
                ];
              };
          }
        );
    };
}
