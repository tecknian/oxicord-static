{
  description = "Minimalist Rust + Ratatui Development Environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.cargo-watch
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.chafa
            pkgs.glib
          ];

          PKG_CONFIG_PATH = "${pkgs.chafa}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.chafa}/lib:${pkgs.glib.out}/lib";
        };
      }
    );
}
