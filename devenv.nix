{
  pkgs,
  inputs,
  ...
}:

{
  packages = [
    pkgs.git
    pkgs.cargo-edit
    pkgs.platformio
    pkgs.zstd

    inputs.bun2nix.packages.${pkgs.system}.default
  ];

  languages.rust.enable = true;
}
