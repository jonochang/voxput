# Voxput Home Manager module
#
# Usage in your flake.nix:
#
#   inputs.voxput.url = "github:jonochang/voxput";
#
#   # Apply the overlay so pkgs.voxput and pkgs.voxputGnomeExtension are available:
#   nixpkgs.overlays = [ inputs.voxput.overlays.default ];
#
#   home-manager.users.alice = { imports = [ inputs.voxput.homeManagerModules.default ]; };
#
# Minimal config:
#
#   services.voxput = {
#     enable = true;
#     apiKeyFile = config.sops.secrets.groq-api-key.path;  # or age.secrets…
#   };
#
# With GNOME extension:
#
#   services.voxput = {
#     enable = true;
#     apiKeyFile = config.sops.secrets.groq-api-key.path;
#     gnome.enable = true;
#   };
#
#   # Enable the extension in GNOME Shell (Home Manager 23.11+):
#   programs.gnome-shell.extensions = [
#     { package = config.services.voxput.gnome.package; }
#   ];

{ config, lib, pkgs, ... }:

let
  cfg = config.services.voxput;
  inherit (lib)
    mkEnableOption mkOption mkIf mkMerge types literalExpression optionalAttrs;
in
{
  options.services.voxput = {
    enable = mkEnableOption "voxput voice-to-text daemon (voxputd)";

    package = mkOption {
      type = types.package;
      default = pkgs.voxput;
      defaultText = literalExpression "pkgs.voxput";
      description = ''
        The voxput package, providing both the `voxput` CLI and the
        `voxputd` daemon binary.  Apply `overlays.default` from the flake
        so `pkgs.voxput` is available, or pass any package you like.
      '';
    };

    apiKeyFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      example = literalExpression "config.sops.secrets.groq-api-key.path";
      description = ''
        Path to a file containing environment variables for the daemon,
        in particular:

          GROQ_API_KEY=gsk_...

        This is passed as `EnvironmentFile` to the systemd user service.
        Use sops-nix, agenix, or any secrets manager that produces a file
        with the above format.

        Alternatively, set the key in `~/.config/voxput/config.toml`:
          [providers.groq]
          api_key = "gsk_..."
      '';
    };

    gnome = {
      enable = mkEnableOption "voxput GNOME Shell extension";

      package = mkOption {
        type = types.package;
        default = pkgs.voxputGnomeExtension;
        defaultText = literalExpression "pkgs.voxputGnomeExtension";
        description = ''
          The GNOME Shell extension package.  Provided by `overlays.default`.
        '';
      };

      shortcut = mkOption {
        type = types.listOf types.str;
        default = [ "<Super>m" ];
        example = literalExpression ''[ "<Super>v" ]'';
        description = ''
          Keyboard shortcut to toggle recording.  Set to `[]` to disable the
          shortcut and rely on the panel menu instead.
        '';
      };

      showNotification = mkOption {
        type = types.bool;
        default = true;
        description = "Show a GNOME notification when transcription completes.";
      };
    };
  };

  config = mkIf cfg.enable (mkMerge [

    # ------------------------------------------------------------------
    # Core: binaries + systemd user service + D-Bus activation
    # ------------------------------------------------------------------
    {
      home.packages = [ cfg.package ];

      systemd.user.services.voxputd = {
        Unit = {
          Description = "Voxput voice dictation daemon";
          Documentation = "https://github.com/jonochang/voxput";
          After = [ "graphical-session.target" ];
        };

        Service = {
          Type = "dbus";
          BusName = "com.github.jonochang.Voxput";
          ExecStart = "${cfg.package}/bin/voxputd";
          Restart = "on-failure";
          RestartSec = 5;
        } // optionalAttrs (cfg.apiKeyFile != null) {
          EnvironmentFile = cfg.apiKeyFile;
        };

        Install = {
          WantedBy = [ "graphical-session.target" ];
        };
      };

      # D-Bus session activation: starts voxputd automatically on first use
      xdg.dataFile."dbus-1/services/com.github.jonochang.Voxput.service".text = ''
        [D-BUS Service]
        Name=com.github.jonochang.Voxput
        Exec=${cfg.package}/bin/voxputd
        SystemdService=voxputd.service
      '';
    }

    # ------------------------------------------------------------------
    # Optional: GNOME Shell extension
    # ------------------------------------------------------------------
    (mkIf cfg.gnome.enable {
      # Make the extension visible to GNOME Shell via XDG_DATA_DIRS.
      # To enable it, add to your config:
      #
      #   programs.gnome-shell.extensions = [
      #     { package = config.services.voxput.gnome.package; }
      #   ];
      #
      # (Home Manager 23.11+; this merges correctly with other extensions.)
      home.packages = [ cfg.gnome.package ];

      # Extension-specific settings (safe to set from a module — these are
      # scoped to the extension's own GSettings schema path)
      dconf.settings."org/gnome/shell/extensions/voxput" = {
        toggle-recording = cfg.gnome.shortcut;
        show-transcript-notification = cfg.gnome.showNotification;
        daemon-auto-start = true;
      };
    })

  ]);
}
