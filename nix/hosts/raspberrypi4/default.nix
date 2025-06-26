{ pkgs, inputs, ... }:
{
  imports = [
    inputs.hardware.nixosModules.raspberry-pi-4

    ./hardware-configuration.nix
  ];

  system.stateVersion = "25.05";
}
