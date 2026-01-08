{
  lib,
  stdenv,
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
  dontUseBunPatch = true;
  dontUseBunCheck = true;

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

  # Handle cross-compilation: binary ends up in target/<triple>/release/ instead of target/release/
  installPhase =
    let
      targetDir =
        if stdenv.hostPlatform != stdenv.buildPlatform then
          "${finalAttrs.cargoRoot}/target/${stdenv.hostPlatform.rust.cargoShortTarget}/release"
        else
          "${finalAttrs.cargoRoot}/target/release";
    in
    ''
      runHook preInstall
      install -Dm755 "${targetDir}/xtee" "$out/bin/xtee"
      runHook postInstall
    '';

  meta = {
    description = "XTEE";
    mainProgram = "xtee";
    homepage = "https://github.com/mossida/xtee";
    platforms = lib.platforms.linux;
    license = lib.licenses.gpl3Only;
    maintainers = with lib.maintainers; [ marcocondrache ];
  };
})
