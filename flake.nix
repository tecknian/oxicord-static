{
  description = "Oxicord - Discord TUI client";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rust-analyzer" "clippy" "rustfmt"];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in {
        packages.default = rustPlatform.buildRustPackage {
          pname = "oxicord";
          version = "0.1.6";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [pkgs.pkg-config pkgs.clang pkgs.mold pkgs.makeBinaryWrapper];

          buildInputs = [
            pkgs.dbus
            pkgs.chafa
            pkgs.glib
          ];

          preCheck = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [pkgs.dbus.lib pkgs.chafa pkgs.glib]}:$LD_LIBRARY_PATH
          '';

          postInstall = ''
            wrapProgram $out/bin/oxicord \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [pkgs.dbus.lib pkgs.chafa pkgs.glib]}"
          '';

          meta = with pkgs.lib; {
            description = "A lightweight, secure Discord terminal client";
            homepage = "https://github.com/linuxmobile/oxicord";
            license = licenses.mit;
            maintainers = [];
            mainProgram = "oxicord";
            platforms = platforms.unix;
          };
        };

        devShells.default = pkgs.mkShell {
          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          packages = [
            rustToolchain
            pkgs.cargo-watch
            pkgs.pkg-config
            pkgs.clang
            pkgs.mold
          ];

          buildInputs = [
            pkgs.dbus
            pkgs.chafa
            pkgs.glib
          ];

          PKG_CONFIG_PATH = "${pkgs.chafa}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig:${pkgs.dbus.dev}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.chafa}/lib:${pkgs.glib.out}/lib:${pkgs.dbus.lib}/lib";
        };
      }
    );
}
