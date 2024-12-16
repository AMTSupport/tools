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
    flake-parts = { url = "github:hercules-ci/flake-parts"; inputs.nixpkgs-lib.follows = "nixpkgs"; };
    # TODO - Remove once devenv updates its lockfile.
    git-hooks = { url = "github:cachix/git-hooks.nix"; inputs.nixpkgs.follows = "nixpkgs"; };
    devenv = { url = "github:cachix/devenv"; inputs.git-hooks.follows = "git-hooks"; };
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
    crane = { url = "github:ipetkov/crane"; };
    nci = { url = "github:yusdacra/nix-cargo-integration"; inputs = { nixpkgs.follows = "nixpkgs"; parts.follows = "flake-parts"; crane.follows = "crane"; }; };
  };

  outputs = inputs@{ flake-parts, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

    imports = [
      inputs.devenv.flakeModule
      inputs.nci.flakeModule
    ];

    debug = true;

    perSystem = { config, system, pkgs, lib, inputs', ... }:
      let
        ourLib = import ./lib.nix { inherit pkgs; };
        craneLib = inputs.crane.mkLib pkgs;

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
        devenv.shells.default = {
          # Fixes https://github.com/cachix/devenv/issues/528
          containers = lib.mkForce { };

          languages.rust = {
            enable = true;
            channel = "nightly";
            # Don't use any components here because we need a combined toolchain
            # so that rust-rover is happy. :)
            components = [ ];

            toolchain = rustToolchain;
          };

          packages = with pkgs; [
            libz
            openssl
            pkg-config

            act
            hyperfine
            cocogitto

            cargo-audit
            cargo-deadlinks
            cargo-deny
            cargo-shear
            cargo-edit
            cargo-expand
            cargo-modules
            cargo-nextest
            cargo-semver-checks
            cargo-unused-features
          ];

          git-hooks = {
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
                extraPackages = [
                  pkgs.openssl
                  pkgs.pkg-config
                ];
              };

              rustfmt = {
                enable = true;
                packageOverrides = {
                  cargo = rustToolchain;
                  rustfmt = rustToolchain;
                };
              };
            };

            settings.rust.check.cargoDeps = pkgs.rustPlatform.importCargoLock {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
          };
        };

        nci = {
          toolchains = {
            mkBuild = _: with inputs'.fenix.packages; combine ([
              minimal.rustc
              minimal.cargo
              complete.clippy
            ] ++ (lib.mapAttrsToList (_: target: targets.${target.rust.rustcTarget}.latest.rust-std) ourLib.buildableTargets));
          };

          projects.tools = {
            path = lib.cleanSourceWith {
              name = "source";
              src = craneLib.path ./.;
              filter = path: type: (craneLib.filterCargoSources path type) || (builtins.match ".*memorable-pass/assets/words.json$" path != null);
            };

            targets = lib.mapAttrs'
              (_: target: lib.nameValuePair target.rust.rustcTarget rec {
                default = target.pkgsCross.targetPlatform.system == pkgs.targetPlatform.system;
                profiles = [ "dev" "release" ];
                depsDrvConfig = rec {
                  env = ourLib.env.mkEnvironment target;

                  mkDerivation = {
                    depsBuildBuild = with target.pkgsCross; [ stdenv.cc ];

                    buildInputs = with target.pkgsCross;
                      lib.optionals (target.pkgsCross.targetPlatform.isLinux && target.pkgsCross.targetPlatform.isx86_64) [ clang mold ]
                      ++ lib.optionals target.pkgsCross.targetPlatform.isLinux [ libz zlib ]
                      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows [ windows.mingw_w64_headers ]
                      ++ lib.optionals (target.pkgsCross.targetPlatform.isWindows && target.pkgsCross.stdenv.isx86_64) [ windows.pthreads ]
                      ++ lib.optionals (!(lib.hasSuffix "clang" env.CC && !target.pkgsCross.targetPlatform.isx86_64)) [ openssl ]; # openssl fails to build on aarch64 (https://github.com/NixOS/nixpkgs/issues/348791)

                    nativeBuildInputs = with pkgs; [ pkg-config ]
                      ++ lib.optionals (!target.pkgsCross.stdenv.buildPlatform.canExecute target.pkgsCross.stdenv.hostPlatform && !target.pkgsCross.targetPlatform.isWindows) [ pkgs.qemu ]
                      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows [ pkgs.wineWow64Packages.minimal ];

                    enableParallelBuilding = true;

                    passthru = {
                      inherit (target.pkgsCross.targetPlatform) system;
                    };
                  };
                };
                drvConfig = depsDrvConfig;
              })
              ourLib.buildableTargets;
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
        } // (lib.trivial.pipe config.nci.outputs [
          # Exclude the empty root crates & crates that are not exported
          (lib.filterAttrs (name: _: name != "tools" && (!(builtins.hasAttr "${name}" config.nci.crates) || config.nci.crates.${name}.export)))
          (lib.mapAttrs' (name: crate: lib.nameValuePair "${name}-allTargets" (pkgs.symlinkJoin {
            inherit name;
            paths = lib.trivial.pipe crate.allTargets [
              (lib.mapAttrsToList (_: target: target.packages.release))
              lib.flatten
            ];
          })))
        ]);

        checks.pre-commit = config.devenv.shells.default.git-hooks.run;
      };
  };
}
