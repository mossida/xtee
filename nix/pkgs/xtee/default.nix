{
  lib,
  rustPlatform,

  bun,
  nodejs,
  cargo-tauri,
  pkg-config,
  moreutils,
  mkBunNodeModules,

  udev,
  libsoup_3,
  openssl,
  webkitgtk_4_1,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "xtee";
  version = "2.0.3";

  src = lib.cleanSource ../../..;

  nodeModules = mkBunNodeModules {
    packages = import ./deps.nix;
  };

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;

  cargoHash = "sha256-GqATnoqSWz9D6mxp19fmkUiX56Vp2mwFXkLhn0D+gks=";

  preBuild = ''
    ln -sf ${finalAttrs.nodeModules}/node_modules ./node_modules
  '';

  nativeBuildInputs = [
    bun
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
