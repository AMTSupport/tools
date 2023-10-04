{ localSystem
, pkgs
, flake-utils
, fenix
, crane
, cocogitto
}:

let
  inherit (pkgs.callPackage ./default.nix { inherit localSystem flake-utils fenix crane; }) passthru;
  fenixPkgs = fenix.packages.${localSystem};
in
(pkgs.mkShell passthru).overrideAttrs (oldAttrs: {
  nativeBuildInputs = with pkgs; [
    act
    hyperfine
    cocogitto.packages.${localSystem}.default
    (fenixPkgs.complete.withComponents [
      "cargo"
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
    cargo-edit
  ] ++ (oldAttrs.nativeBuildInputs or [ ]);
})
