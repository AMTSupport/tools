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
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv.url = "github:cachix/devenv";
    pre-commit-hooks-nix = { url = "github:cachix/pre-commit-hooks.nix"; inputs.nixpkgs.follows = "nixpkgs"; };
    crane = { url = "github:ipetkov/crane"; };
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
    # Unpin when https://github.com/yusdacra/nix-cargo-integration/issues/159 is resolved
    nci = { url = "github:yusdacra/nix-cargo-integration"; };
  };

  outputs = inputs@{ self, flake-parts, crane, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    imports = [
      inputs.pre-commit-hooks-nix.flakeModule
      inputs.devenv.flakeModule
      inputs.nci.flakeModule
    ];

    systems = [ "x86_64-linux" ];

    debug = true;

    perSystem = { config, system, pkgs, lib, ... }:
      let
        # outputTargets = [
        #   "x86_64-unknown-linux-gnu"
        #   "aarch64-unknown-linux-gnu"
        #   #          "x86_64-apple-darwin" # error: don't yet have a `targetPackages.darwin.LibsystemCross for x86_64-apple-darwin`
        #   #          "aarch64-apple-darwin" # can't compile from linux to darwin
        #   "x86_64-pc-windows-gnu"
        #   "aarch64-pc-windows-gnullvm"
        # ];

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
        ]); #(map (target: fenixPkgs.targets.${target}.latest.rust-std) outputTargets));

        # makeLinkerEnv = isNative: crossPkgs: target:
        #   let
        #     linker =
        #       if isNative
        #       then "${crossPkgs.clang}/bin/${crossPkgs.clang.targetPrefix}clang"
        #       else let inherit (crossPkgs.stdenv) cc; in "${cc}/bin/${cc.targetPrefix}cc";
        #   in
        #   {
        #     # "CC" = linker;
        #     "CC_${target}" = linker;
        #     "CARGO_TARGET_${target}_LINKER" = linker;

        #     "CARGO_TARGET_${target}_RUSTFLAGS" =
        #       if isNative && crossPkgs.stdenv.targetPlatform.isLinux
        #       then "-C link-arg=-fuse-ld=${lib.getExe crossPkgs.mold}"
        #       else null;

        #     "CARGO_TARGET_${target}_RUNNER" =
        #       if isNative
        #       then null
        #       else if crossPkgs.stdenv.targetPlatform.isWindows
        #       then
        #         pkgs.writeScript "wine-wrapper" ''
        #           #!${lib.getExe pkgs.bash}
        #           export WINEPREFIX="$(mktemp -d)"
        #           exec ${(lib.getExe (pkgs.wine.override { wineBuild = "wine64"; }))} $@
        #         ''
        #       else "${pkgs.qemu}/bin/qemu-${crossPkgs.targetPlatform.qemuArch}";
        #   };
      in
      {
        pre-commit.settings.hooks = {
          actionlint.enable = true;

          # Rust specific hooks
          cargo-check.enable = true;
          clippy.enable = true;
          rustfmt.enable = true;

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
            openssl
            clang
            mold
            libz

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
            cargo-xwin # Required for building windows aarch targets.
          ] ++ config.pre-commit.settings.enabledPackages;

          env = /*makeLinkerEnv true pkgs "X86_64_UNKNOWN_LINUX_GNU" //*/ {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
              openssl
              clang
              mold
            ]);
          };
        };

        packages = lib.trivial.pipe ourLib.workspaceOutputs [
          (builtins.map (craneOutputs: craneOutputs.default))
          (builtins.map (package: lib.nameValuePair package.pname package))
          lib.listToAttrs
        ];

        # nci = rec {
        #   # toolchains = {
        #   #   build = rustToolchain;
        #   #   shell = rustToolchain;
        #   # };

        #   projects.tools = {
        #     path = self;
        #     export = true;

        #     profiles = {
        #       dev = { };
        #       release = { };
        #     };

        #     targets = lib.trivial.pipe outputTargets [
        #       (map (target:
        #         let
        #           envTarget = builtins.replaceStrings [ "-" ] [ "_" ] (lib.toUpper target);
        #           nixTarget = builtins.replaceStrings [ "-gnullvm" "-gnu" "-pc" "-unknown" "-apple" ] [ "" "" "" "" "" ] target;
        #           split = lib.strings.splitString "-" nixTarget;
        #           arch = builtins.elemAt (split) 0;
        #           platform = builtins.elemAt (split) 1;

        #           isNative = system == nixTarget;
        #           isDarwin = platform == "apple";
        #           isWindows = platform == "windows";
        #           isLinux = platform == "linux";
        #         in
        #         lib.nameValuePair target rec {
        #           # Default if native
        #           default = isNative;
        #           # No dev profile for non-native targets
        #           profiles = if default then (builtins.attrNames projects.tools.profiles) else [ "release" ];

        #           depsDrvConfig =
        #             let
        #               crossPackages =
        #                 if isNative then pkgs
        #                 else if nixTarget == "x86_64-linux" then pkgs.pkgsCross.gnu64
        #                 else if platform == "windows" then pkgs.pkgsCross.mingwW64
        #                 else if platform == "darwin" then pkgs.pkgsCross.${nixTarget}
        #                 else if arch == "aarch64" then pkgs.pkgsCross.mingwW64
        #                 else throw "Unsupported target: ${nixTarget}";
        #             in
        #             {
        #               mkDerivation = {
        #                 # Make sure some windows build dependencies are available.
        #                 depsBuildBuild = (lib.optionals crossPackages.targetPlatform.isWindows [
        #                   crossPackages.stdenv.cc
        #                   crossPackages.windows.mingw_w64_pthreads
        #                   crossPackages.windows.pthreads
        #                 ]);

        #                 # Common build inputs
        #                 buildInputs = [ pkgs.openssl ];
        #                 nativeBuildInputs = [ pkgs.pkg-config ];

        #                 passthru = {
        #                   inherit arch platform isNative isDarwin isWindows isLinux;
        #                 };
        #               };

        #               env = makeLinkerEnv default crossPackages envTarget;
        #             };

        #           drvConfig = depsDrvConfig;
        #         }))
        #       builtins.listToAttrs
        #     ];
        #   };
        # };

        # packages =
        #   let
        #     allCratesTargetsReleases = lib.trivial.pipe config.nci.outputs [
        #       (lib.filterAttrs (name: _: name != "tools"))
        #       lib.attrValues
        #       (builtins.map (crate: lib.attrValues crate.allTargets))
        #       lib.flatten
        #     ];

        #     # craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        #   in
        #   {
        #     all = pkgs.symlinkJoin {
        #       name = "all";
        #       description = "Compile all crates for all targets";
        #       paths = builtins.map (target: target.packages.release) allCratesTargetsReleases;
        #     };

        #     # test = craneLib.buildPackage {
        #     #   cname = "test";
        #     #   pname = "test";
        #     #   version = "0.2.0";
        #     #   src = craneLib.path ./.;
        #     #   strictDeps = false;
        #     #   doCheck = false;
        #     #   cargoLock = craneLib.path ./Cargo.lock;

        #     #   cargoExtraArgs = "--package lib";
        #     #   CARGO_BUILD_TARGET = "aarch64-pc-windows-gnullvm";

        #     # };
        #   } // (lib.trivial.pipe config.nci.outputs [
        #     (lib.filterAttrs (name: _: name != "tools"))
        #     lib.attrValues
        #     (builtins.map (crate: lib.attrValues crate.allTargets))
        #     lib.flatten
        #     (builtins.map (target: target.packages.release))
        #     (builtins.filter (package: !package.out.passthru.isNative))
        #     (builtins.map (package: lib.nameValuePair "${package.name}-${package.out.passthru.arch}-${package.out.passthru.platform}" package))
        #     lib.listToAttrs
        #   ])/* // (lib.trivial.pipe allCratesTargetsReleases [ # Work around for adding build support for arm on windows, because there is no native build support on Nix for it.
        #     # Filter to the native target for each crate
        #     (builtins.filter (package: package.out.passthru.isNative))
        #     # From the native target get the information for creating a derivation from runCommand
        #     (builtins.map (package: lib.nameValuePair "${package.name}-${package.out.passthru.arch}-${package.out.passthru.platform}" ))

        #     ])*/ // (lib.trivial.pipe [ "windows" "darwin" "linux" ] [
        #     (map (platform: lib.nameValuePair "all-${platform}" (pkgs.symlinkJoin {
        #       name = "all-${platform}";
        #       description = "Compile all crates for the ${platform} target";
        #       paths = lib.trivial.pipe allCratesTargetsReleases [
        #         (builtins.filter (package: package.out.passthru.platform == platform))
        #         (builtins.map (target: target.packages.release))
        #       ];
        #     })))
        #     lib.listToAttrs
        #   ]);
      };
  };
}
