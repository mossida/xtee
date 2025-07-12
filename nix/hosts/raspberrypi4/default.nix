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
    program = pkgs.writeScriptBin "xtee" ''
      #!/usr/bin/env bash

      export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/gsettings-desktop-schemas-45.0"
      export GIO_MODULE_DIR="${pkgs.glib-networking}/lib/gio/modules/"

      exec ${outputs.packages.aarch64-linux.xtee}/bin/xtee
    '';
  };

  # Workaround for https://github.com/NixOS/nixpkgs/issues/154163
  nixpkgs.overlays = [
    (_: prev: { makeModulesClosure = x: prev.makeModulesClosure (x // { allowMissing = true; }); })
  ];

  system.stateVersion = "25.05";
}
