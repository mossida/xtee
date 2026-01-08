{
  pkgs ? import <nixpkgs> { },
  bun2nix,
  ...
}:
{
  xtee = pkgs.callPackage ./xtee { bun2nix = bun2nix.packages.${pkgs.system}.default; };
}
