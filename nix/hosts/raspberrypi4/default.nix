{
  pkgs,
  inputs,
  outputs,
  ...
}:
{
  imports = [
    inputs.hardware.nixosModules.raspberry-pi-4

    ./hardware-configuration.nix
  ];

  boot.initrd.systemd.emergencyAccess = true;

  users.users.xtee = {
    isNormalUser = true;
    useDefaultShell = true;
  };

  powerManagement.enable = false;

  services.cage = {
    enable = true;
    user = "xtee";
    program = "${outputs.packages.aarch64-linux.xtee}/bin/xtee";
  };

  # Workaround for https://github.com/NixOS/nixpkgs/issues/154163
  nixpkgs.overlays = [
    (_: prev: { makeModulesClosure = x: prev.makeModulesClosure (x // { allowMissing = true; }); })
  ];

  system.stateVersion = "25.05";
}
