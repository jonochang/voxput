# record

Record audio from the microphone and transcribe via the Groq Whisper API.
This is a one-shot command that does not require the daemon.

## Usage

```bash
voxput record [OPTIONS]
```

## Examples

```bash
# Record until any key is pressed (default), print to stdout
voxput record

# Record until keypress OR 10 seconds, whichever comes first
voxput record --duration 10

# Copy result to clipboard instead
voxput record --output clipboard

# Both stdout and clipboard
voxput record --output both

# Force English transcription
voxput record --language en

# Use a specific Whisper model
voxput record --model whisper-large-v3

# Print transcript as JSON
voxput record --json

# Use a specific input device
voxput record --device "USB Audio"
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `--duration` | no limit | Stop after N seconds (or keypress, whichever first) |
| `--output` | `stdout` | Output target: `stdout`, `clipboard`, or `both` |
| `--language` | auto | Language hint for transcription (e.g., `en`, `fr`) |
| `--model` | `whisper-large-v3-turbo` | Whisper model to use |
| `--device` | system default | Audio input device name |
| `--json` | off | Print transcript as JSON |
