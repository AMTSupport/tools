{
  description = "Tools Rust Project";

  nixConfig = {
    extra-substituters = [
      "https://amt.cachix.org"
      "https://racci.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "amt.cachix.org-1:KiJsXTfC7rGJc4DmNlLA56caUUWuc8YsOfzpPgredJI="
      "racci.cachix.org-1:Kl4opLxvTV9c77DpoKjUOMLDbCv6wy3GVHWxB384gxg="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
     ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv.url = "github:cachix/devenv";
    nci.url = "github:yusdacra/nix-cargo-integration";
    pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs@{ self, flake-parts, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    imports = [
      inputs.pre-commit-hooks-nix.flakeModule
      inputs.devenv.flakeModule
      inputs.nci.flakeModule
    ];

    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

    perSystem = { config, system, pkgs, lib, ... }: let
      rustToolchain = let fenixPkgs = inputs.fenix.packages.${system}; in fenixPkgs.combine [
        fenixPkgs.complete.cargo
        fenixPkgs.complete.rustc
        fenixPkgs.complete.rust-src
        fenixPkgs.complete.rust-analyzer
        fenixPkgs.complete.clippy
        fenixPkgs.complete.rustfmt
      ];

      useMold = isNative: pkgs: envTarget: lib.mkIf (isNative && pkgs.stdenv.targetPlatform.isLinux) (let
        clang = "${pkgs.clang}/bin/${pkgs.clang.targetPrefix}clang";
      in {
        "CC" = clang;
        "CC_${envTarget}" = clang;
        "CARGO_TARGET_${envTarget}_LINKER" = clang;
        "CARGO_TARGET_${envTarget}_RUSTFLAGS" = "-C link-arg=-fuse-ld=${lib.getExe pkgs.mold}";
      });
    in rec {
      pre-commit.settings.hooks = {
        actionlint.enable = true;

        # Rust specific hooks
        cargo-check = {
          enable = true;
          package = rustToolchain;
          packageOverrides = {
            cargo = rustToolchain;
          };
        };
        clippy = {
          enable = true;
          packageOverrides = {
            cargo = rustToolchain;
            clippy = rustToolchain;
          };
        };
        rustfmt = {
          enable = true;
          package = rustToolchain;
          packageOverrides = {
            cargo = rustToolchain;
            rustfmt = rustToolchain;
          };
        };

        check-case-conflicts.enable = true;
        check-docstring-first.enable = true;
        check-toml.enable = true;
        check-vcs-permalinks.enable = true;
        check-yaml.enable = true;
        deadnix.enable = true;
        detect-private-keys.enable = true;
        editorconfig-checker.enable = true;
        end-of-file-fixer.enable = true;
        markdownlint.enable = true;
        mdl.enable = true;
        mdsh.enable = true;
#        mixed-line-ending.enable = true;
        nil.enable = true;
        nixpkgs-fmt.enable = true;
        statix.enable = true;
        tagref.enable = true;
        typos.enable = true;
      };

      devenv.shells.default = rec {
        difftastic.enable = true;

        languages.rust = {
          enable = true;
          channel = "nightly";
          # Don't use any components here because we need a combined toolchain
          # so that rust-rover is happy. :)
          components = [ ];
        };

        packages = with pkgs; [
          openssl
          clang
          mold

          pkg-config
          rustToolchain

          act
          hyperfine
          cocogitto
          cargo-udeps
          cargo-audit
          cargo-expand
          cargo-nextest
          cargo-cranky
          cargo-edit
        ] ++ config.pre-commit.settings.enabledPackages;

        env = useMold true pkgs "X86_64_UNKNOWN_LINUX_GNU" // {
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            openssl
            clang
            mold
          ]);
        };
      };

      nci = rec {
        toolchains = {
          build = rustToolchain;
          shell = devenv.shells.default;
        };

        projects.tools = {
          path = self;
          export = true;

          profiles = {
            dev = { };
            release = {
              runTests = true;
            };
          };

          targets = lib.trivial.pipe [
            "unknown-linux-gnu"
            "pc-windows-gnu"
#            "apple-darwin"
          ] [
            (map (target: [
              "x86_64-${target}"
            ] ++ lib.optionals (target != "pc-windows-gnu") [ "aarch64-${target}" ]))
            lib.flatten
            (map (target: let
              envTarget = builtins.replaceStrings [ "-" ] [ "_" ] (lib.toUpper target);
              nixTarget = builtins.replaceStrings [ "-gnu" "-pc" "-unknown" "-apple" ] [ "" "" "" "" ] target;
            in lib.nameValuePair target rec {
              # Default if native
              default = system == nixTarget;
              # No dev profile for non-native targets
              profiles = if default then (builtins.attrNames projects.tools.profiles) else [ "release" ];
              # Only run tests on native target
              # profiles.release.runTests = default;

              depsDrvConfig = let
                crossPackages = if default then pkgs
                  else if nixTarget == "x86_64-linux" then pkgs.pkgsCross.gnu64
                  else if nixTarget == "x86_64-windows" then pkgs.pkgsCross.mingwW64
                  else if nixTarget == "aarch64-linux" then pkgs.pkgsCross.aarch64-multiplatform
                  else pkgs.pkgsCross.${nixTarget};
              in {
                deps.stdenv = crossPackages.clangStdenv;

                mkDerivation = {
                  # Make sure some windows build dependencies are available.
                  depsBuildBuild = lib.optionals crossPackages.targetPlatform.isWindows (with crossPackages; [
                    stdenv.cc
                    windows.mingw_w64_pthreads
                    windows.pthreads
                  ]);

                  # Common build inputs
                  buildInputs = [ pkgs.openssl ];
                  nativeBuildInputs = [ pkgs.pkg-config ];
                };

                # Use mold linker for linux targets
                env = useMold default pkgs envTarget;
              };
            }))
            builtins.listToAttrs
          ];
        };
      };

      # Create a symlink to all binaries for each crate
      # Create a symlink to all binaries for each target
      packages = {
        all = pkgs.symlinkJoin {
          name = "all";
          description = "Compile all crates for all targets";
          paths = lib.trivial.pipe config.nci.outputs [
            (lib.filterAttrs (name: _: name != "tools"))
            lib.attrValues
            (builtins.map (crate: lib.attrValues crate.allTargets))
            lib.flatten
            (builtins.map (target: target.packages.release))
          ];
        };

        allNative = pkgs.symlinkJoin {
          name = "allNative";
          description = "Compile all crates for the native target";
          paths = lib.trivial.pipe config.nci.outputs [
            (lib.filterAttrs (name: _: name != "tools"))
            lib.attrValues
            (builtins.map (crate: crate.packages.release))
          ];
        };

        allTargets = lib.mapAttrs (name: crate: {
          all = pkgs.symlinkJoin {
            inherit name;
            description = "Compile all targets for ${name}";
            paths = lib.trivial.pipe crate.allTargets [
              lib.attrValues
              (builtins.map (target: target.packages.release))
            ];
          };
        }) config.nci.outputs;
      };
    };
  };
}
