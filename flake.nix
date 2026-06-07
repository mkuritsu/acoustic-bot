{
  description = "rust flake template";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs = {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    eachSystem = fn: nixpkgs.lib.genAttrs (import systems) (system: fn nixpkgs.legacyPackages.${system});
  in {
    nixosModules.default = import ./nix/nixosModule.nix self;

    packages = eachSystem (pkgs: rec {
      default = acoustic-bot;
      acoustic-bot = pkgs.callPackage ./nix/package.nix {};
    });

    devShells = eachSystem (pkgs: {
      default = pkgs.callPackage ./nix/shell.nix {};
    });
  };
}
