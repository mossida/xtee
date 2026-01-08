{
  pkgs ? import <nixpkgs> { },
  bun2nix,
  ...
}:
{
  xtee = pkgs.callPackage ./xtee { inherit (bun2nix.packages.${pkgs.system}.default) fetchBunDeps; };
}
