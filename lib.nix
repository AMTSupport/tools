{ pkgs }:
let
  inherit (pkgs) lib;
in
rec {
  buildableTargets = lib.trivial.pipe targets [
    (lib.filterAttrs (_: target: target.canBuild))
  ];

  targets = {
    Linux-X86_64 = rec {
      pkgsCross = pkgs.pkgsCross.gnu64;
      canBuild = true;
      inherit (pkgsCross.targetPlatform) rust;
    };

    Apple-X86_64 = rec {
      pkgsCross = pkgs.pkgsCross.x86_64-darwin;
      canBuild = pkgs.targetPlatform.isDarwin;
      inherit (pkgsCross.targetPlatform) rust;
    };

    Windows-X86_64 = rec {
      pkgsCross = pkgs.pkgsCross.mingwW64;
      canBuild = true;
      inherit (pkgsCross.targetPlatform) rust;
    };

    Linux-Aarch64 = rec {
      pkgsCross = pkgs.pkgsCross.aarch64-multiplatform;
      canBuild = true;
      inherit (pkgsCross.targetPlatform) rust;
    };

    Apple-Aarch64 = rec {
      pkgsCross = pkgs.pkgsCross.aarch64-darwin;
      canBuild = pkgs.targetPlatform.isDarwin;
      inherit (pkgsCross.targetPlatform) rust;
    };

    Windows-Aarch64 = rec {
      pkgsCross = pkgs.pkgsCross.ucrtAarch64;
      canBuild = true;
      rust = pkgsCross.targetPlatform.rust // rec {
        rustcTarget = "aarch64-pc-windows-gnullvm";
        rustcTargetSpec = rustcTarget;

        cargoShortTarget = rustcTarget;
        cargoEnvVarTarget = "AARCH64_PC_WINDOWS_GNULLVM";
      };
    };
  };

  env = rec {
    # TODO Use clang for all linux builds - openssl fails to build on aarch64 (https://github.com/NixOS/nixpkgs/issues/348791)
    getCCForTarget = target: if target.pkgsCross.targetPlatform.isLinux && target.pkgsCross.targetPlatform.system == pkgs.targetPlatform.system
      then "${target.pkgsCross.clang.targetPrefix}clang"
      else "${target.pkgsCross.stdenv.cc.targetPrefix}cc";

    getRustFlagsForTarget = target: if target.pkgsCross.targetPlatform.isLinux && target.pkgsCross.targetPlatform.system == pkgs.targetPlatform.system
      then "-C link-arg=-fuse-ld=${lib.getExe pkgs.mold}"
      else null;

    getRunnerForTarget = target:
      if target.pkgsCross.stdenv.buildPlatform.canExecute target.pkgsCross.stdenv.hostPlatform
      then null
      else if target.pkgsCross.targetPlatform.isWindows
      then pkgs.writeScript "wine-wrapper" ''
        #!${pkgs.stdenv.shell}
        export WINEPREFIX="$(mktemp -d)"
        exec ${lib.getExe pkgs.wineWow64Packages.minimal} $@
      ''
      else pkgs.lib.getExe' pkgs.qemu "qemu-${target.pkgsCross.targetPlatform.qemuArch}";

    mkEnvironment = target: rec {
      CC = getCCForTarget target;
      TARGET_CC = CC;
      "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_LINKER" = TARGET_CC;
      "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_RUSTFLAGS" = getRustFlagsForTarget target;
      "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_RUNNER" = getRunnerForTarget target;
    };
  };
}
