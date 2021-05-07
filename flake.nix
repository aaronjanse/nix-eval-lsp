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
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;
          cargoSha256 = "sha256-5J/joDCZ4U8AQgTV91pa+vDJLQlPGWLWHu/rnNxI8Zc=";
          RUSTC_BOOTSTRAP = 1;
        }) { };
      defaultPackage = self.packages.${system}.nix-eval-lsp;
    }
  );
}
