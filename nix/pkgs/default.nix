{
  pkgs ? import <nixpkgs> { },
  bun2nix,
  ...
}:
rec {
  xtee = pkgs.callPackage ./xtee { inherit (bun2nix.lib.${pkgs.system}) mkBunNodeModules; };
}
