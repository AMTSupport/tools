{
  description = "Tools Rust Project";

  nixConfig = {
    extra-substituters = [
      "https://amt.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "amt.cachix.org-1:KiJsXTfC7rGJc4DmNlLA56caUUWuc8YsOfzpPgredJI="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv.url = "github:cachix/devenv";
    pre-commit-hooks-nix = { url = "github:cachix/pre-commit-hooks.nix"; inputs.nixpkgs.follows = "nixpkgs"; };
    crane = { url = "github:ipetkov/crane"; };
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
    nci = { url = "github:yusdacra/nix-cargo-integration"; };
  };

  outputs = inputs@{ flake-parts, crane, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" ];

    imports = [
      inputs.pre-commit-hooks-nix.flakeModule
      inputs.devenv.flakeModule
      inputs.nci.flakeModule
    ];

    debug = true;

    perSystem = { config, system, pkgs, lib, ... }:
      let
        ourLib = import ./lib.nix {
          inherit pkgs;
          fenixPkgs = inputs.fenix.packages.${system};
          craneLib = inputs.crane.mkLib pkgs;
        };

        rustToolchain = let fenixPkgs = inputs.fenix.packages.${system}; in fenixPkgs.combine ([
          fenixPkgs.complete.cargo
          fenixPkgs.complete.rustc
          fenixPkgs.complete.rust-src
          fenixPkgs.complete.rust-analyzer
          fenixPkgs.complete.clippy
          fenixPkgs.complete.rustfmt
        ] ++ (lib.mapAttrsToList (_: target: fenixPkgs.targets.${target.rust.rustcTarget}.latest.rust-std) ourLib.buildableTargets));
      in
      {
        pre-commit.settings = {
          excludes = [
            "CHANGELOG.md$"
            "^flake.lock$"
            "^crates/memorable-pass/assets/words.json$"
          ];

          hooks = {
            actionlint.enable = true;
            check-case-conflicts.enable = true;
            check-docstring-first.enable = true;
            check-toml.enable = true;
            check-vcs-permalinks.enable = true;
            check-yaml.enable = true;
            deadnix.enable = true;
            detect-private-keys.enable = true;
            editorconfig-checker.enable = true;
            end-of-file-fixer.enable = true;
            nil.enable = true;
            nixpkgs-fmt.enable = true;
            statix.enable = true;
            typos.enable = true;

            clippy = {
              enable = true;
              package = rustToolchain;
            };

            rustfmt = {
              enable = true;
              packageOverrides = {
                cargo = rustToolchain;
                rustfmt = rustToolchain;
              };
            };

            cargo-check = {
              enable = true;
              package = rustToolchain;
            };
          };
        };

        devenv.shells.default = {
          # Fixes https://github.com/cachix/devenv/issues/528
          containers = lib.mkForce { };

          difftastic.enable = true;

          languages.rust = {
            enable = true;
            channel = "nightly";
            # Don't use any components here because we need a combined toolchain
            # so that rust-rover is happy. :)
            components = [ ];

            toolchain = {
              cargo = rustToolchain;
              rustfmt = rustToolchain;
              clippy = rustToolchain;
            };
          };

          packages = with pkgs; [
            libz
            openssl
            pkg-config
            rustToolchain

            act
            hyperfine
            cocogitto
            cargo-udeps
            cargo-audit
            cargo-deny
            cargo-expand
            cargo-nextest
            cargo-cranky
            cargo-edit
            cargo-machete
            cargo-deadlinks
            cargo-unused-features
            cargo-hack
            cargo-modules
            cargo-geiger
            cargo-audit
            cargo-bloat
            cargo-diet
          ] ++ config.pre-commit.settings.enabledPackages;
        };

        nci = {
          toolchains = {
            build = rustToolchain;
            shell = rustToolchain;
          };

          projects.tools = {
            path = ./.;

            targets = lib.mapAttrs'
              (_: target: lib.nameValuePair target.rust.rustcTarget {
                default = target.pkgsCross.targetPlatform.system == pkgs.targetPlatform.system;
                profiles = [ "dev" "release" ];
                drvConfig = {
                  mkDerivation = {
                    depsBuildBuild = lib.optionals (!target.pkgsCross.stdenv.buildPlatform.canExecute target.pkgsCross.stdenv.hostPlatform && !target.pkgsCross.targetPlatform.isWindows) (with pkgs; [ qemu ])
                      ++ lib.optionals (target.pkgsCross.targetPlatform.isWindows && target.pkgsCross.stdenv.isx86_64) (with target.pkgsCross; [ windows.mingw_w64_pthreads windows.pthreads ]);

                    buildInputs = lib.optionals target.pkgsCross.targetPlatform.isLinux (with target.pkgsCross; [ openssl clang mold ])
                      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows (with target.pkgsCross; [ windows.mingw_w64_headers ]);

                    nativeBuildInputs = with pkgs; [ pkg-config openssl ]
                      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows [ pkgs.wine64 ];

                    passthru = {
                      inherit (target.pkgsCross.targetPlatform) system;
                    };
                  };

                  env = ourLib.environmentForTarget target;
                };
              })
              ourLib.targets;
          };
        };

        packages = (lib.trivial.pipe config.nci.outputs [
          (lib.filterAttrs (name: _: name != "tools"))
          (lib.mapAttrsToList (_: crate: lib.attrsToList crate.allTargets))
          lib.flatten
          (builtins.map (attr: lib.attrsToList attr.value.packages))
          lib.flatten
          (lib.map (attr: lib.nameValuePair "${attr.value.name}-${attr.value.out.passthru.system}-${attr.name}" attr.value))
          lib.listToAttrs
        ]) // {
          release = pkgs.symlinkJoin {
            name = "all-release";
            description = "Compile all crates for all targets";
            paths = builtins.map (target: target.packages.release) (lib.flatten (builtins.map (crate: lib.attrValues crate.allTargets) config.nci.outputs));
          };
        };
      };
  };
}
