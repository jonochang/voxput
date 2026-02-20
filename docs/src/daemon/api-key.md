# API Key Management

The daemon needs a Groq API key to transcribe audio. There are several ways
to provide it, depending on your setup.

## Option A: Manual config file (simplest)

Create `~/.config/voxput/config.toml` outside of Nix:

```toml
[providers.groq]
api_key = "gsk_..."
```

The daemon reads this file at startup. Keep the file out of version control.

## Option B: systemd user environment

```bash
systemctl --user set-environment GROQ_API_KEY=gsk_...
systemctl --user restart voxputd
```

Add to your shell profile or `~/.config/environment.d/groq.conf` to persist
across reboots.

## Option C: Secrets manager (`apiKeyFile`)

Pass a file whose contents are shell-style environment variable assignments:

```
GROQ_API_KEY=gsk_...
```

### With sops-nix

```nix
sops.secrets.groq-api-key = { sopsFile = ./secrets.yaml; };
services.voxput.apiKeyFile = config.sops.secrets.groq-api-key.path;
```

### With agenix

```nix
age.secrets.groq-api-key = { file = ./secrets/groq-api-key.age; };
services.voxput.apiKeyFile = config.age.secrets.groq-api-key.path;
```

## Security note

Do not put your API key directly in your Nix config. It will end up in the
world-readable Nix store and in your git history.
