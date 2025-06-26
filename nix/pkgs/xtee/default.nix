{
  lib,
  stdenv,
  rustPlatform,
  fetchFromGitHub,
  nodejs,
  bun,
  wrapGAppsHook3,
  cargo,
  rustc,
  cargo-tauri_1,
  pkg-config,
  esbuild,
  buildGoModule,
  libayatana-appindicator,
  gtk3,
  webkitgtk_4_0,
  libsoup_2_4,
  openssl,
  xdotool,
}:

stdenv.mkDerivation (finalAttrs: {
  pname = "xtee";
  version = "3.0.7";

  src = fetchFromGitHub {
    owner = "mossida";
    repo = "xtee";
    tag = finalAttrs.version;
    hash = "sha256-0Q1hf1AGAZv6jt05tV3F6++lzLpddvjhiykIhV40cPs=";
  };

  cargoRoot = "src-tauri";
  buildAndTestSubdir = "src-tauri";

  cargoDeps = rustPlatform.fetchCargoVendor {
    inherit (finalAttrs)
      pname
      version
      src
      cargoRoot
      ;
    hash = "sha256-dyXINRttgsqCfmgtZNXxr/Rl8Yn0F2AVm8v2Ao+OBsw=";
  };

  nativeBuildInputs = [
    rustPlatform.cargoSetupHook
    cargo
    rustc
    cargo-tauri.hook
    nodejs
    bun
    wrapGAppsHook3
    pkg-config
  ];

  buildInputs = [
    gtk3
    libsoup_2_4
    libayatana-appindicator
    openssl
    webkitgtk_4_0
    xdotool
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
