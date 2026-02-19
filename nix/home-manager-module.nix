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
# Minimal config (API key inline — fine for non-shared machines):
#
#   services.voxput = {
#     enable = true;
#     apiKey  = "gsk_...";   # or use apiKeyFile for secrets managers
#   };
#
# With a secrets manager (sops-nix / agenix):
#
#   services.voxput = {
#     enable     = true;
#     apiKeyFile = config.sops.secrets.groq-api-key.path;
#   };
#
# With GNOME extension:
#
#   services.voxput = {
#     enable = true;
#     apiKey = "gsk_...";
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
    mkEnableOption mkOption mkIf mkMerge types literalExpression
    optionalAttrs optionalString;

  # Build the config.toml text from module options.
  # Only sections/keys that are set are emitted so the file stays minimal.
  configToml = ''
    provider = "groq"

    [providers.groq]
    ${optionalString (cfg.apiKey != null) ''api_key = "${cfg.apiKey}"''}
    ${optionalString (cfg.model  != null) ''model   = "${cfg.model}"''}

    [audio]
    ${optionalString (cfg.device != null) ''device = "${cfg.device}"''}
  '';

  hasInlineConfig = cfg.apiKey != null || cfg.model != null || cfg.device != null;
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

    # ------------------------------------------------------------------
    # API key — choose ONE of the three options below
    # ------------------------------------------------------------------

    apiKey = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "gsk_xxxxxxxxxxxxxxxxxxxx";
      description = ''
        Groq API key written directly into
        `~/.config/voxput/config.toml`.  Convenient for personal machines
        where the key does not need to be kept out of the Nix store.

        For stricter security use `apiKeyFile` instead (the key is then
        loaded from a runtime secret that never enters the store).

        Setting either `apiKey`, `model`, or `device` causes the module
        to manage `~/.config/voxput/config.toml` for you.
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

        This is passed as `EnvironmentFile` to the systemd user service
        so the key is loaded at runtime and never enters the Nix store.
        Use sops-nix, agenix, or any secrets manager that produces a file
        in the above format.
      '';
    };

    model = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "whisper-large-v3";
      description = ''
        Whisper model name to use.  Defaults to `whisper-large-v3-turbo`
        (fast, high quality).  Written into the managed config.toml when
        set.
      '';
    };

    device = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "default";
      description = ''
        ALSA/PipeWire device name for microphone input.  Leave `null` to
        use the system default.  Written into the managed config.toml when
        set.
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
    # Optional: managed config.toml
    # Generated when apiKey, model, or device is set inline.
    # ------------------------------------------------------------------
    (mkIf hasInlineConfig {
      xdg.configFile."voxput/config.toml".text = configToml;
    })

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
