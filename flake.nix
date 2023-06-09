{
  description = "dev-shell for tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in with pkgs; {
        devShells.default = mkShell {
          packages = [ pkgs.bashInteractive ];
          buildInputs = [
            openssl
            mold
            pkg-config
            gcc_multi
            gcc
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              extensions = [ "rust-src" ];
              targets = [ "x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu" ];
            }))
          ];
        };
      });
}

