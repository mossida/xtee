{
  lib,
  pkgs,
  inputs,
  outputs,
  ...
}:
{
  imports = with inputs.pi.nixosModules; [
    raspberry-pi-4.base
    raspberry-pi-4.display-vc4
  ];

  users.users.xtee = {
    isNormalUser = true;
    useDefaultShell = true;
    extraGroups = [
      "wheel"
      "systemd-journal"
      "dialout"
      "networkmanager"
      "video"
    ];

    initialHashedPassword = "";
  };

  services.getty.autologinUser = "xtee";

  networking = {
    hostName = "xtee";
    wireless = {
      enable = false;
    };
  };

  powerManagement.enable = false;

  environment.sessionVariables = {
    # https://github.com/tauri-apps/tauri/issues/7354
    XDG_DATA_DIRS = lib.mkMerge [
      "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}"
    ];

    GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules";
  };

  services.cage = {
    enable = true;
    user = "xtee";
    program = "${outputs.packages.aarch64-linux.xtee}/bin/xtee";
    environment = {
      WLR_DPI = "192";
      XTEE_LOG = "info";
    };
  };

  system.stateVersion = "25.05";
}
