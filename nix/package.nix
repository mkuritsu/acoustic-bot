{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  libopus,
  ffmpeg,
  yt-dlp,
  makeBinaryWrapper
}:
rustPlatform.buildRustPackage {
  pname = "acoustic-bot";
  version = "0.1.0";
  cargoHash = "sha256-NGeN8zNbmLQxosLk5tn0p6LckouyCg1tqkq+Rmgr6/Y=";

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
    ];
  };

  nativeBuildInputs = [
    makeBinaryWrapper
    pkg-config
  ];

  buildInputs = [
    openssl
    libopus
  ];

  postInstall = ''
    wrapProgram $out/bin/acoustic-bot \
      --prefix PATH : ${lib.makeBinPath [ ffmpeg yt-dlp ]} \
  '';

  meta.mainProgram = "acoustic-bot";
}
