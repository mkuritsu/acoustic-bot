{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  libopus,
  ffmpeg,
  yt-dlp,
}:
rustPlatform.buildRustPackage {
  pname = "acoustic-bot";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
    ];
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
    libopus
    ffmpeg
    yt-dlp
  ];

  cargoHash = "sha256-NGeN8zNbmLQxosLk5tn0p6LckouyCg1tqkq+Rmgr6/Y=";

  meta.mainProgram = "acoustic-bot";
}
