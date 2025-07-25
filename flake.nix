{
  description = "chos";
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    fenix = {
      url = "github:nix-community/fenix/main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils/main";
    };
  };
  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        limine = pkgs.limine.override {
          enableAll = true;
          buildCDs = true;
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            fenix.packages.${system}.complete.toolchain
            pkgs.qemu_full
            limine
            pkgs.gum
          ];
          shellHook = ''
            export LIMINE_PATH=${limine}/share/limine
          '';
        };
      }
    );
}
