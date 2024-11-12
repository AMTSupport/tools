{ pkgs, fenixPkgs, craneLib }:
let
  inherit (pkgs) lib;

  src = craneLib.path ./.;
in
rec {
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

  hasSubCrates = cargoToml: builtins.length (cargoToml.workspace.members or [ ]) >= 1;

  # TODO Support for non default members
  getWorkspaceCrates = cargoToml: onlyDefault:
    if (!onlyDefault)
    then
      lib.trivial.pipe (builtins.attrNames (builtins.readDir (src + "/crates"))) [
        (builtins.map (crate: src + "/crates/${crate}/Cargo.toml"))
      ]
    else
      lib.trivial.pipe (cargoToml.workspace.default-members) [
        (builtins.map (src: src + "/${src}/Cargo.toml"))
      ];

  createPackages = cargoToml:
    let
      inherit (craneLib.crateNameFromCargoToml { inherit src cargoToml; }) pname version;
    in
    lib.trivial.pipe (lib.attrsToList targets) [
      (builtins.filter (target: target.value.canBuild))
      (builtins.map (target: pkgs.callPackage ./crate.nix {
        cname = pname;
        hasBinary = (builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"))).bin or [ ] != [ ];

        hostPkgs = pkgs;
        target = target.value;
        systemSuffix = target.name;

        inherit src pname version craneLib fenixPkgs;
      }))
    ];

  workspaceOutputs =
    let
      cargoTomlPath = craneLib.path ./Cargo.toml;
      cargoToml = builtins.fromTOML (builtins.readFile cargoTomlPath);
    in
    if hasSubCrates cargoToml
    then
      let workspaceCrates = getWorkspaceCrates cargoToml false; in
      lib.trivial.pipe workspaceCrates [
        (builtins.map (crateToml: createPackages crateToml))
        lib.flatten
      ]
    else [ createPackages cargoTomlPath ];
}
