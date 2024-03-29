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

    systems.url = "github:nix-systems/default-linux";
    flake-utils = { url = "github:numtide/flake-utils"; inputs.systems.follows = "systems"; };

    crane = { url = "github:ipetkov/crane"; inputs.nixpkgs.follows = "nixpkgs"; };
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };

    cocogitto = { url = "github:DaRacci/cocogitto"; inputs.nixpkgs.follows = "nixpkgs"; };
    nix-config = { url = "github:DaRacci/nix-config"; inputs = {
        flake-utils = { url = "github:numtide/flake-utils"; inputs.systems.follows = "systems"; };
        fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
        cocogitto = { url = "github:DaRacci/cocogitto"; inputs.nixpkgs.follows = "nixpkgs"; };
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, fenix, cocogitto, nix-config, ... }@inputs:
    let
      # TODO - Darwin support (error: don't yet have a `targetPackages.darwin.LibsystemCross for x86_64-apple-darwin`)
      targets = [ "x86_64-linux" "x86_64-windows" ];
      onAll = localSystem: f: (builtins.foldl' (attr: target: attr // (f target)) { } targets);
    in
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        pkgs = import nixpkgs { system = localSystem; };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        hasSubCrates = (builtins.length (cargoToml.workspace.members or [ ])) >= 1;

        cargoOutputs = onAll localSystem
          (crossSystem:
            let
              disambiguate = name: if crossSystem == localSystem then name else "${name}-${crossSystem}";
            in
            (if hasSubCrates then
              let
                members = cargoToml.workspace.default-members or [ ];
                getCargoToml = path: builtins.fromTOML (builtins.readFile (./. + "/${path}" + "/Cargo.toml"));
                memberName = path: let cargo = getCargoToml path; in cargo.package.name;
                getPkg = workspace: pkgs.callPackage ./default.nix { inherit self localSystem crossSystem flake-utils crane fenix workspace; };
              in
              if builtins.length members >= 1 then
                builtins.listToAttrs (builtins.map (member: { name = disambiguate (memberName member); value = let split = builtins.split "/" member; in getPkg (builtins.elemAt split (builtins.length split - 1)); }) members)
              else
                builtins.listToAttrs (builtins.map (member: { name = disambiguate (memberName "crates/${member}"); value = getPkg member; }) (builtins.attrNames (builtins.readDir ./crates)))
            else { }) // (if ((cargoToml.package.name or null) == null) then { } else (builtins.listToAttrs [{ name = disambiguate "default"; value = pkgs.callPackage ./default.nix { inherit localSystem crossSystem flake-utils crane fenix; }; }]))
          );
      in
      {
        packages = builtins.mapAttrs (name: outputs: outputs.crateBinary) cargoOutputs // {
          default = pkgs.symlinkJoin {
            name = "all";
            paths = builtins.attrValues (builtins.mapAttrs (name: outputs: outputs.crateBinary) cargoOutputs);
          };
        };

        devShells.default = nix-config.devShells.${localSystem}.rust-nightly;

        checks =
          let
            nativeOutputs = builtins.filter (o: o.isNative) (builtins.attrValues (builtins.mapAttrs (name: output: {
              inherit name;
              inherit (output.passthru) isNative;

              inherit (output) crateFmt crateClippy crateTest;
            }) cargoOutputs));
          in
          builtins.foldl' (attr: packageChecks: (attr // packageChecks)) { } (builtins.map (crate: {
            "${crate.name}-formatting" = crate.crateFmt;
            "${crate.name}-lint" = crate.crateClippy;
            "${crate.name}-test" = crate.crateTest;
          }) nativeOutputs);
      });
}
