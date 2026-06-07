self: {
  config,
  lib,
  pkgs,
  ...
}: let
  system = pkgs.stdenv.hostPlatform.system;

  cfg = config.services.acoustic-bot;
  pkg = self.packages.${system}.default;
in {
  options.services.acoustic-bot = {
    enable = lib.mkEnableOption "Enable acoustic discord bot service";

    envFile = lib.mkOption {
      type = lib.types.path;
      description = "Path to a file containing the env vars to use, including BOT_TOKEN";
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups.acoustic-bot = {};

    users.users.acoustic-bot = {
      isSystemUser = true;
      group = "acoustic-bot";
    };

    systemd.services.acoustic-bot = {
      enable = true;
      after = ["network.target"];
      wantedBy = ["default.target"];
      description = "Acoustic-bot systemd service";
      serviceConfig = {
        Type = "simple";
        User = "acoustic-bot";
        Restart = "on-failure";
        EnvironmentFile = cfg.envFile;
        ExecStart = lib.getExe pkg;
      };
    };
  };
}
