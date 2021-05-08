{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let pkgs = nixpkgs.legacyPackages.${system}; in {
      packages.nix-eval-lsp = pkgs.callPackage
        ({ stdenv, lib, rustPlatform, fetchFromGitHub }:
        rustPlatform.buildRustPackage {
          pname = "nix-eval-lsp";
          version = "0.1.0";
          src = ./.;
          cargoSha256 = "sha256-OoHGx9RLWahJ11z9EmnnJDj/b2GJhQgJslTmSVuxc7Y=";
          RUSTC_BOOTSTRAP = 1;
        }) { };
      defaultPackage = self.packages.${system}.nix-eval-lsp;
    }
  );
}
