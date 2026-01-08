{
  lib,
  rustPlatform,

  bun,
  nodejs,
  cargo-tauri,
  pkg-config,
  moreutils,
  bun2nix,

  udev,
  libsoup_3,
  openssl,
  webkitgtk_4_1,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "xtee";
  version = "2.0.3";

  src = lib.cleanSource ../../..;

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = import ./deps.nix;
  };

  dontUseBunBuild = true;
  dontUseBunPatch = false;

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;

  cargoHash = "sha256-5TZ5AI1YCRRYZ8ZUMDfXjPzh75IMTS13t9Y2Ctre1mA=";

  nativeBuildInputs = [
    bun
    bun2nix.hook
    moreutils
    cargo-tauri.hook
    pkg-config
    nodejs
  ];

  buildInputs = [
    udev
    libsoup_3
    openssl
    webkitgtk_4_1
  ];

  meta = {
    description = "XTEE";
    mainProgram = "xtee";
    homepage = "https://github.com/mossida/xtee";
    platforms = lib.platforms.linux;
    license = lib.licenses.gpl3Only;
    maintainers = with lib.maintainers; [ marcocondrache ];
  };
})
