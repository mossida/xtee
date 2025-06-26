{
  pkgs ? import <nixpkgs> { },
  ...
}:
rec {
  xtee = pkgs.callPackage ./xtee { };
}
