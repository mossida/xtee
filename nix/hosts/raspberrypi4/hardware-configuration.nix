{
  boot = {
    kernelParams = [
      "vt.global_cursor_default=0"
      "cma=256M"
      "quiet"
      "loglevel=3"
      "udev.log_level=3"
      "rd.systemd.show_status=auto"
    ];

    loader.timeout = 0;
  };

  fileSystems = {
    "/" = {
      device = "/dev/disk/by-label/NIXOS_SD";
      fsType = "ext4";
      options = [ "noatime" ];
    };

    "/boot/firmware" = {
      device = "/dev/disk/by-label/FIRMWARE";
      fsType = "vfat";
      options = [
        "nofail"
        "noauto"
      ];
    };
  };

  hardware.raspberry-pi."4" = {
    gpio.enable = true;
    fkms-3d.enable = true;
    touch-ft5406.enable = true;
  };

  hardware.deviceTree.overlays = [
    {
      name = "gpio-shutdown";
      dtsText = ''
        /dts-v1/;
        /plugin/;

        / {
            compatible = "brcm,bcm2711";

            fragment@0 {
                target = <&gpio>;
                __overlay__ {
                    pin_state: shutdown_button_pins {
                        brcm,pins     = <3>;
                        brcm,function = <0>;
                        brcm,pull     = <1>;
                    };
                };
            };

            fragment@1 {
                target-path = "/soc";
                __overlay__ {
                    shutdown_button {
                        compatible    = "gpio-keys";
                        pinctrl-names = "default";
                        pinctrl-0     = <&pin_state>;
                        status        = "okay";

                        button: shutdown {
                            label              = "shutdown";
                            linux,code         = <116>;
                            gpios              = <&gpio 3 0>;
                            debounce-interval  = <100>;
                        };
                    };
                };
            };
        };
      '';
    }
  ];

  nixpkgs.hostPlatform.system = "aarch64-linux";
}
