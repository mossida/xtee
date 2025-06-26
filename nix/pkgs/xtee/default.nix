{
  lib,
  rustPlatform,
  fetchFromGitHub,

  moreutils,
  bun,
  nodejs,
  cargo-tauri,
  pkg-config,
  wrapGAppsHook3,

  libsoup_3,
  openssl,
  webkitgtk_4_1,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "xtee";
  version = "2.0.2";

  src = fetchFromGitHub {
    owner = "mossida";
    repo = "xtee";
    tag = finalAttrs.version;
    hash = "sha256-0Q1hf1AGAZv6jt05tV3F6++lzLpddvjhiykIhV40cPs=";
  };

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;

  cargoHash = "sha256-dyXINRttgsqCfmgtZNXxr/Rl8Yn0F2AVm8v2Ao+OBsw=";

  nativeBuildInputs = [
    moreutils
    bun
    nodejs
    cargo-tauri.hook
    pkg-config
    wrapGAppsHook3
  ];

  buildInputs = [
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
