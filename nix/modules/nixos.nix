{ config, lib, pkgs, inputs, ... }:
with lib;
let
  cfg = config.services.immich-background-tool;
in
{
  options.services.immich-background-tool = {
    enable = mkEnableOption "Immich Background tool";

    secretFile = mkOption {
      example = "/private/immich-data";
      default = null;
      description = ''
        env format file that contains information about your immich endpoint
      '';
    };

    package = mkOption {
      description = "the package to use";
      type = types.package;
      default = pkgs.callPackage ./rust.nix;
    };
  };

  config = mkIf cfg.enable {

    systemd.user = {
      services.immich-background-tool = {
        description = "Immich Background tool service";

        serviceConfig = {
          Type = "oneshot";
          ExecStart = "${cfg.package}/bin/immich-background-tool ${cfg.secretFile}";
        };
      };
      timers.immich-background-tool = {
        wantedBy = [ "timers.target" ];
        requires = [ "sway-session.target" ];
        wants = [ "graphical-session.target" ];
        timerConfig = {
          OnCalendar = "*/5 * * * *";
          Persistent = true;
        };
      };
    };
  };
}
