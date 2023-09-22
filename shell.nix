{ localSystem
, pkgs
, flake-utils
, fenix
, crane
}:

let
  mainPkg = pkgs.callPackage ./default.nix { inherit localSystem flake-utils fenix crane; };
  fenixPkgs = fenix.packages.${localSystem};
in
mainPkg.overrideAttrs (oa: {
  nativeBuildInputs = with pkgs; [
    cocogitto
    (fenixPkgs.complete.withComponents [
      "rust-src"
      "rust-analyzer"
      "clippy-preview"
      "rustfmt-preview"
    ])
    cargo-udeps
    cargo-audit
    cargo-expand
    cargo-nextest
    cargo-expand
    cargo-cranky
  ] ++ (oa.nativeBuildInputs or [ ]);
})
