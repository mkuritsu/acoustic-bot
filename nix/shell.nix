{pkgs, ...}:
pkgs.mkShell {
  packages = with pkgs; [
    rustup
    pkg-config
    libopus
    openssl
    ffmpeg
    yt-dlp
  ];
}
