{
  fileSystems = {
    "/boot" = {
      device = "/dev/disk/by-label/BOOT";
      fsType = "vfat";
    };

    "/firmware" = {
      device = "/dev/disk/by-label/FIRMWARE";
      fsType = "vfat";
    };
  };

  nixpkgs.hostPlatform.system = "aarch64-linux";
}
