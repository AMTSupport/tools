{ src
, cname
, pname
, version
, hasBinary

, craneLib
, fenixPkgs

, hostPkgs
, target
}:
let
  inherit (hostPkgs) lib;

  toolchain = fenixPkgs.combine [
    fenixPkgs.targets.${target.rust.cargoShortTarget}.latest.rust-std
    fenixPkgs.complete.rustc
    fenixPkgs.complete.cargo
  ];
  craneLibWithToolchain = craneLib.overrideToolchain toolchain;
  isNative = hostPkgs.targetPlatform.system == target.pkgsCross.targetPlatform.system;

  commonEnvironment = {
    CARGO_BUILD_TARGET = target.rust.cargoShortTarget;
  } // (lib.optionalAttrs (!isNative) {
    "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_RUNNER" = hostPkgs.lib.getExe' hostPkgs.qemu "qemu-${target.pkgsCross.targetPlatform.qemuArch}";
  }) // (lib.optionalAttrs target.pkgsCross.targetPlatform.isLinux rec {
    "CC" = "${hostPkgs.clang}/bin/${target.pkgsCross.clang.targetPrefix}clang";
    "CC_${target.rust.cargoEnvVarTarget}" = CC;
    "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_LINKER" = CC;
    "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_RUSTFLAGS" = "-C link-arg=-fuse-ld=${lib.getExe hostPkgs.mold}";
  }) // (lib.optionalAttrs target.pkgsCross.targetPlatform.isWindows {
    "CARGO_TARGET_${target.rust.cargoEnvVarTarget}_RUNNER" = hostPkgs.writeScript "wine-wrapper" ''
      #!${hostPkgs.stdenv.shell}
      export WINEPREFIX="$(mktemp -d)"
      exec ${hostPkgs.wine.override { wineBuild = "wine64"; }}/bin/wine64 $@
    '';
  });

  commonDeps = {
    depsBuildBuild = [ target.pkgsCross.stdenv.cc ]
      ++ lib.optionals (!isNative && !target.pkgsCross.targetPlatform.isWindows) (with hostPkgs; [ qemu ])
      ++ lib.optionals (target.pkgsCross.targetPlatform.isWindows && target.pkgsCross.stdenv.isx86_64) (with target.pkgsCross; [ windows.mingw_w64_pthreads windows.pthreads ]);

    buildInputs = lib.optionals target.pkgsCross.targetPlatform.isLinux (with target.pkgsCross; [ target.pkgsCross.openssl clang mold ])
      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows (with target.pkgsCross; [ windows.mingw_w64_headers ]);

    nativeBuildInputs = with hostPkgs; [ pkg-config ]
      ++ lib.optionals target.pkgsCross.targetPlatform.isWindows [ (pkgs.wine.override { wineBuild = "wine64"; }) ];

    LD_LIBRARY_PATH = lib.makeLibraryPath (with hostPkgs; [
      openssl
    ] ++ lib.optionals target.pkgsCross.targetPlatform.isLinux (with target.pkgsCross; [
      wayland
      libxkbcommon
      vulkan-loader
      libglvnd
      egl-wayland
      wayland-protocols
      xwayland
      libdecor
    ]));
  };

  commonArgs = commonDeps // commonEnvironment // {
    inherit cname version src;
    pname = "${pname}-${target.pkgsCross.targetPlatform.system}";

    cargoLock = craneLib.path ./Cargo.lock;
    cargoExtraArgs = "--package ${cname}";

    strictDeps = true;
    doCheck = false; # Checks are done with flake
  };

  cargoArtifact = craneLibWithToolchain.buildDepsOnly commonArgs;

  cargoBuild = artifact: extra: craneLibWithToolchain.buildPackage (commonArgs // {
    cargoArtifacts = artifact;
  } // (if extra != null then extra commonArgs else { }));

  cargoClippy = artifact: extra: craneLibWithToolchain.cargoClippy (commonArgs // {
    cargoArtifacts = artifact;
    cargoClippyExtraArgs = "--package ${cname} -- --deny warnings";
  } // (if extra != null then extra commonArgs else { }));

  cargoFmt = artifact: extra: craneLibWithToolchain.cargoFmt (commonArgs // {
    cargoArtifacts = artifact;
  } // (if extra != null then extra commonArgs else { }));

  cargoTest = artifact: extra: craneLibWithToolchain.cargoNextest (commonArgs // {
    cargoArtifacts = artifact;
  } // (if extra != null then extra commonArgs else { }));
in
rec {
  passthru = commonDeps // commonEnvironment // {
    inherit isNative target;
  };

  artifact = cargoArtifact;

  library = cargoBuild cargoArtifact (args: {
    cargoExtraArgs = "${args.cargoExtraArgs} --lib";
  });

  test = cargoTest cargoArtifact null;
  clippy = cargoClippy cargoArtifact null;
  format = cargoFmt cargoArtifact null;

  default = library;
} // (lib.optionalAttrs hasBinary rec {
  executable = cargoBuild cargoArtifact (args: {
    cargoExtraArgs = "--bin ${args.cname}";

    # Add a suffix to the binary name to avoid conflicts.
    postInstall = ''
      mv $out/bin/${args.cname} \
        $out/bin/${args.pname}${target.pkgsCross.targetPlatform.extensions.executable}
    '';
  });

  default = executable;
})
