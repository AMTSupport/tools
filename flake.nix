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

  outputs = inputs@{ flake-parts, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

    imports = [
      inputs.pre-commit-hooks-nix.flakeModule
      inputs.devenv.flakeModule
      inputs.nci.flakeModule
    ];

    debug = true;

    perSystem = { config, system, pkgs, lib, inputs', ... }:
      let
        ourLib = import ./lib.nix { inherit pkgs; };

        rustToolchain = with inputs'.fenix.packages; combine ([
          complete.cargo
          complete.rustc
          complete.rust-src
          complete.rust-analyzer
          complete.clippy
          complete.rustfmt
        ] ++ (lib.mapAttrsToList (_: target: targets.${target.rust.rustcTarget}.latest.rust-std) ourLib.buildableTargets));

        rootCargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        defaultMembers = rootCargoToml.workspace.default-members or [ ];
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
              packageOverrides = {
                cargo = rustToolchain;
                clippy = rustToolchain;
              };
            };

            rustfmt = {
              enable = true;
              packageOverrides = {
                cargo = rustToolchain;
                rustfmt = rustToolchain;
              };
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
            mkBuild = _: with inputs'.fenix.packages; combine ([
              minimal.rustc
              minimal.cargo
            ] ++ (lib.mapAttrsToList (_: target: targets.${target.rust.rustcTarget}.latest.rust-std) ourLib.buildableTargets));
          };

          projects.tools = {
            path = ./.;
            export = false;

            targets = lib.mapAttrs'
              (_: target: lib.nameValuePair target.rust.rustcTarget rec {
                default = target.pkgsCross.targetPlatform.system == pkgs.targetPlatform.system;
                profiles = [ "dev" "release" ];
                depsDrvConfig = {
                  mkDerivation = {
                    depsBuildBuild = [ target.pkgsCross.stdenv.cc ];

                    buildInputs = with target.pkgsCross; lib.optionals target.pkgsCross.targetPlatform.isx86_64 [ target.pkgsCross.openssl ] # FIXME OpenSSL for aarch64 fails to build with clang (https://github.com/NixOS/nixpkgs/issues/348791)
                      ++ lib.optionals target.pkgsCross.targetPlatform.isLinux (with target.pkgsCross; [ libz clang mold ])
                      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows (with target.pkgsCross; [ windows.mingw_w64_headers ])
                      ++ lib.optionals (target.pkgsCross.targetPlatform.isWindows && target.pkgsCross.stdenv.isx86_64) (with target.pkgsCross; [ windows.pthreads ]);

                    nativeBuildInputs = with pkgs; [ pkg-config ]
                      ++ lib.optionals (!target.pkgsCross.stdenv.buildPlatform.canExecute target.pkgsCross.stdenv.hostPlatform && !target.pkgsCross.targetPlatform.isWindows) [ pkgs.qemu ]
                      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows [ pkgs.wineWow64Packages.minimal ];


                    passthru = {
                      inherit (target.pkgsCross.targetPlatform) system;
                    };
                  };

                  env = ourLib.env.mkEnvironment target;
                };
                drvConfig = depsDrvConfig;
              })
              ourLib.buildableTargets;
          };

          crates = {
            lib.export = false;
            macros.export = false;
          };
        };

        packages = (lib.trivial.pipe config.nci.outputs [
          # Exclude the empty root crates & crates that are not exported
          (lib.filterAttrs (name: _: name != "tools" && (!(builtins.hasAttr "${name}" config.nci.crates) || config.nci.crates.${name}.export)))
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
            paths = lib.trivial.pipe config.packages [
              # Include all release packages
              (lib.filterAttrs (name: _: lib.hasSuffix "-release" name))
              # Only include packages defined as default-members
              (lib.filterAttrs (name: _:
                let
                  parts = lib.splitString "-" name;
                  crateName = builtins.elemAt parts 0;
                  directoryName = "crates/${crateName}";
                in
                builtins.elem directoryName defaultMembers))
              builtins.attrValues
            ];
          };
        };

        # TODO - Refactor once https://github.com/cachix/git-hooks.nix/pull/396 is merged
        checks.pre-commit = pkgs.lib.mkForce (
          let
            drv = config.pre-commit.settings.run;
          in
          pkgs.stdenv.mkDerivation {
            name = "pre-commit-run";
            src = config.pre-commit.settings.rootSrc;
            buildInputs = [ pkgs.git pkgs.openssl pkgs.pkg-config ];
            nativeBuildInputs = [ pkgs.rustPlatform.cargoSetupHook ];
            cargoDeps = pkgs.rustPlatform.importCargoLock {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
            buildPhase = drv.buildCommand;
          }
        );
      };
  };
}
