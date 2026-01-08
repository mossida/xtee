{
  description = "XTEE";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    systems.url = "github:nix-systems/default-linux";
    hardware.url = "github:nixos/nixos-hardware";

    bun2nix.url = "github:baileyluTCD/bun2nix";
    bun2nix.inputs.nixpkgs.follows = "nixpkgs";

    pi.url = "github:nvmd/nixos-raspberrypi";
  };

  outputs =
    {
      self,
      pi,
      nixpkgs,
      systems,
      bun2nix,
      ...
    }@inputs:
    let
      inherit (self) outputs;

      lib = nixpkgs.lib;
      forEachSystem = f: lib.genAttrs (import systems) (system: f pkgsFor.${system});
      pkgsFor = lib.genAttrs (import systems) (
        system:
        import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        }
      );
    in
    {

      packages = forEachSystem (pkgs: import ./nix/pkgs { inherit pkgs bun2nix; });

      nixosConfigurations = {
        raspberrypi4 = pi.lib.nixosSystem {
          modules = [
            ./nix/hosts/raspberrypi4
          ];
          specialArgs = {
            inherit inputs outputs;
          };
        };
      };
    };
}
