{
  description = "Only a dev shell for now";
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs = {
      nixpkgs.follows = "nixpkgs";
      flake-utils.follows = "flake-utils";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.stable."1.66.1".minimal.override {
              extensions = [
                "rust-src" # for rust-analyzer
                "llvm-tools-preview" # for coverage
                "clippy"
              ];
            })

            jdk17
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.rustfmt))
            rust-analyzer

            cargo-nextest
            cargo-insta
            cargo-llvm-cov
          ];

          INSTA_TEST_RUNNER = "nextest";
        };
      }
    );
}
