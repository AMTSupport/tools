{
  description = "dev-shell for tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix, ... }:
    let
#      cargoToml = builtins.fromToml (builtins.readFile ./Cargo.toml);
#      name = cargoToml.package.name;
    in
    flake-utils.lib.eachDefaultSystem (system: let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;

        toolchain = with fenix.packages.${system}; combine [
          minimal.cargo
          minimal.rustc
          complete.rust-src
          targets.x86_64-pc-windows-gnu.latest.rust-std
          targets.x86_64-unknown-linux-gnu.latest.rust-std
        ];

        naersk-lib = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };

        buildPackage = target: { nativeBuildInputs ? [ ], ...}@args:
          naersk-lib.buildPackage({
              name = "tools";
              src = ./.;
              doCheck = false;
              strictDeps = true;
              release = false;
            } // (lib.optionalAttrs (target != system) {
              CARGO_BUILD_TARGET = target;
            }) // args // {
              nativeBuildInputs = with pkgs; [
#                fenix.complete.rustfmt-preview
              ] ++ nativeBuildInputs;
            }
          );

#
#        buildWindows = target: { nativeBuildInputs ? [ ], ...}@args:
#          buildPackage "x86_64-pc-windows-gnu" {
#            doCheck = false;#system == "x86_64-linux";
#
#            depsBuildBuild = with pkgs; [
#              pkgsCross.mingwW64.stdenv.cc
#              pkgsCross.mingwW64.windows.pthreads
#            ];
#
#            nativeBuildInputs = lib.optional doCheck pkgs.wineWowPackages.stable;
#
#            CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
#            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = pkgs.writeScript "wine-wrapper" ''
#              # Without this, wine will error out when attempting to create the
#              # prefix in the build's homeless shelter.
#              export WINEPREFIX="$(mktemp -d)"
#              exec wine64 $@
#            '';
#          };
      in rec {
        packages = {
          # TODO :: Default run/build from system arch
          default = buildPackage system { };

          backup.x86_64-unknown-linux-gnu = buildPackage "x86_64-unknown-linux-gnu" {
            nativeBuildInputs = with pkgs; [ openssl pkgsStatic.stdenv.cc mold ];
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          };

          backup.x86_64-pc-windows-gnu = buildPackage "x86_64-pc-windows-gnu" {
            doCheck = false;#system == "x86_64-linux";

            depsBuildBuild = with pkgs; [
              pkgsCross.mingwW64.stdenv.cc
              pkgsCross.mingwW64.buildPackages.gcc
              pkgsCross.mingwW64.windows.pthreads
            ];

#            nativeBuildInputs = lib.optional doCheck pkgs.wineWowPackages.stable;

            CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = pkgs.writeScript "wine-wrapper" ''
              # Without this, wine will error out when attempting to create the
              # prefix in the build's homeless shelter.
              export WINEPREFIX="$(mktemp -d)"
              exec wine64 $@
            '';
          };
        };


        devShells.default = pkgs.mkShell {
          packages = [ pkgs.bashInteractive ];
          nativeBuildInputs = with pkgs; [
            license-cli
            cargo
          ];
        };
      });
}

