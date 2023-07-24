{
  description = "Learn how Cargo and other build tools/package managers work under the hood by building one";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            git
            just
            (rust-bin.stable."1.70.0".default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = ["x86_64-unknown-linux-gnu"];
            })
          ];
          RUST_SRC_PATH = "${rust-bin.stable."1.70.0".default}/lib/rustlib/src/rust/library";
        };
      }
    );
}
