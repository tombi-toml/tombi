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

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );

        # All the files in the source directory
        unfilteredRoot = ./.;
        src = nixpkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = nixpkgs.lib.fileset.unions [
            # Default files from crane (Rust and Cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Also keep any JSON files
            (nixpkgs.lib.fileset.fileFilter (file: file.hasExt "json") unfilteredRoot)
          ];
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
