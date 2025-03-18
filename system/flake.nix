{
  description = "XTEE";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    nixos-hardware.url = "github:NixOS/nixos-hardware/master";

    nixos-generators.url = "github:nix-community/nixos-generators";
    nixos-generators.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixos-hardware,
      nixos-generators,
    }:
    {
      packages.aarch64-linux = {
        iso = nixos-generators.nixos-generate {
          system = "aarch64-linux";
          format = "iso";
          modules = [
            nixos-hardware.nixosModules.raspberry-pi-4
          ];
        };
      };
    };
}
