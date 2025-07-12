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
        ovmf = {
          riscv64 =
            (import nixpkgs {
              inherit system;
              crossSystem = {
                config = "riscv64-unknown-none-elf";
              };
            }).pkgs.OVMF;
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            fenix.packages.${system}.complete.toolchain
            pkgs.qemu_full
            limine
            pkgs.just
            ovmf.riscv64
            pkgs.ninja
            pkgs.meson
            pkgs.mesonlsp
          ];
          shellHook = ''
            export LIMINE_PATH=${limine}/share/limine
            export OVMF_RISCV64_PATH=${ovmf.riscv64}
          '';
        };
      }
    );
}
