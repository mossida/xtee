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
  installPhase = ''
    runHook preInstall

    # Find the binary in the appropriate target directory
    local binPath
    if [ -n "''${CARGO_BUILD_TARGET:-}" ]; then
      binPath="${finalAttrs.cargoRoot}/target/$CARGO_BUILD_TARGET/release/xtee"
    else
      binPath="${finalAttrs.cargoRoot}/target/release/xtee"
    fi

    install -Dm755 "$binPath" "$out/bin/xtee"

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
