{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustpkg = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rustfmt" "clippy"]; # rust-src for rust-analyzer
          targets = ["x86_64-unknown-linux-gnu"];
        };
      in
        with pkgs; {
          devShells = {
            default = mkShell {
              buildInputs = [
                openssl
                pkg-config
                rust-analyzer
                rustpkg
                libusb1
              ];
            };
          };
        }
    );
}
