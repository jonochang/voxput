# devices

List available audio input devices.

## Usage

```bash
voxput devices [--json]
```

## Examples

```bash
# Plain text output with default device marked
voxput devices

# JSON output
voxput devices --json
```

The JSON output is an array of objects with `name` and `is_default` fields.
