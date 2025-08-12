{
  lib,
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
    password = "xtee";
    extraGroups = [
      "wheel"
      "systemd-journal"
      "dialout"
    ];
  };

  networking = {
    hostName = "xtee";

    interfaces = {
      end0 = {
        useDHCP = false;
        ipv4.addresses = [
          {
            address = "10.10.0.1";
            prefixLength = 24;
          }
        ];
      };
    };

    firewall = {
      allowedTCPPorts = [ 22 ];
      allowPing = true;
    };
  };

  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "no";
      PasswordAuthentication = true;
      KbdInteractiveAuthentication = false;
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
      XTEE_LOG = "debug";
    };
  };

  # Workaround for https://github.com/NixOS/nixpkgs/issues/154163
  nixpkgs.overlays = [
    (_: prev: { makeModulesClosure = x: prev.makeModulesClosure (x // { allowMissing = true; }); })
  ];

  system.stateVersion = "25.05";
}
