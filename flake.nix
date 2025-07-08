{
  description = "Bounded vector development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rust-bin.stable.latest.default
          cargo-hack
        ];
      };

      packages.${system}.check = pkgs.writeShellScriptBin "check" ''
        cargo hack check --feature-powerset --no-dev-deps --exclude-features=nightly
        cargo hack test --each-feature --exclude-features=nightly
        cargo fmt --all -- --check --color always        
        cargo clippy --all-features --exclude-features=nightly -- -D warnings
      '';
    };
}
