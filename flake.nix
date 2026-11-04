{
  description = "Nix-flake development environment for tombi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils = {
      url = "github:numtide/flake-utils";
    };

    crane = {
      url = "github:ipetkov/crane";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      self,
      crane,
      flake-utils,
      nixpkgs,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        toolVersions = (builtins.fromTOML (builtins.readFile ./mise.toml)).tools;
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.${toolVersions.rust.version}.latest.default.override {
            extensions = toolVersions.rust.components;
            targets = toolVersions.rust.targets;
          }
        );

        # All the files in the source directory
        unfilteredRoot = ./.;
        # Crane's mkDummySrc walks the original source behind filtered-source wrappers.
        # Materialize the fileset first so it only sees the selected build sources.
        src = builtins.path {
          name = "tombi-source";
          path = nixpkgs.lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = nixpkgs.lib.fileset.unions [
              # Default files from crane (Rust and Cargo files)
              (nixpkgs.lib.fileset.difference
                (craneLib.fileset.commonCargoSources unfilteredRoot)
                # Crane's dummy-source generation expands these test fixture paths as globs.
                (nixpkgs.lib.fileset.unions [
                  (./crates/tombi-linter/tests/fixtures/cargo/workspace-members/bracket + "[1]/Cargo.toml")
                  (./crates/tombi-linter/tests/fixtures/cargo/workspace-members/bracket + "[1]/src/main.rs")
                ])
              )
              # Also keep any JSON files
              (nixpkgs.lib.fileset.fileFilter (file: file.hasExt "json") unfilteredRoot)
              # Classifier data used by the pyproject extension
              ./extensions/tombi-extension-pyproject/data
              # Suite pin used by include_str! in json-schema-test
              ./json-schema-test/src/SUITE_PIN
            ];
          };
        };

        tombiPkg = craneLib.buildPackage {
          inherit src;

          pname = "tombi";
          # Can't run 'cargo test' after build since nix
          # doesn't allow for network calls in sandbox
          doCheck = false;
          doNotSign = false;
        };
      in
      {
        checks = {
          inherit tombiPkg;
        };

        packages.default = tombiPkg;

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks
          checks = self.checks.${system};

          packages = with pkgs; [
            openssl
            pkg-config
          ];
        };
      }
    );
}
