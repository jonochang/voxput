Scenarios

These are behavior-focused scenarios written in Given/When/Then form. They cover the current CLI MVP and key future flows described in the specs.

CLI MVP

1. Record to stdout (default)
Given a user has a working microphone and `GROQ_API_KEY` is set
When they run `voxput record`
Then the tool waits until a key is pressed to stop recording and prints the transcript to stdout

2. Record with custom duration
Given a user has a working microphone and `GROQ_API_KEY` is set
When they run `voxput record --duration 10`
Then the tool records audio for 10 seconds and prints the transcript to stdout

3. Record with language hint
Given a user speaks French and has `GROQ_API_KEY` set
When they run `voxput record --language fr`
Then the transcript is returned in French when possible

4. Record to clipboard
Given a user has a working clipboard in their desktop session and `GROQ_API_KEY` is set
When they run `voxput record --output clipboard`
Then the transcript is copied to the clipboard and not printed to stdout

5. Record to stdout and clipboard
Given a user has a working clipboard and `GROQ_API_KEY` is set
When they run `voxput record --output both`
Then the transcript is printed to stdout and copied to the clipboard

6. Record with device selection
Given a user has multiple audio input devices
When they run `voxput record --device "USB Audio"`
Then audio is captured from the named device

7. List devices (plain text)
Given a user has one or more audio input devices
When they run `voxput devices`
Then the tool prints a list of device names and marks the default device

8. List devices (JSON)
Given a user has one or more audio input devices
When they run `voxput devices --json`
Then the tool prints a JSON array of device objects with `name` and `is_default`

9. Missing API key
Given `GROQ_API_KEY` is not set and no API key is in config
When the user runs `voxput record`
Then the command fails with a clear message instructing how to set the API key

10. Provider error
Given the Groq API is unreachable or returns an error
When the user runs `voxput record`
Then the command fails with a user-friendly error message that includes the HTTP status

Future (daemon + indicator)

11. Push-to-talk success
Given the background daemon is running and the GNOME extension is installed
When the user holds the push-to-talk hotkey and speaks
Then the indicator shows Recording, then Transcribing, and the transcript appears at the cursor

12. Push-to-talk network failure
Given the daemon is running but the network is down
When the user records via push-to-talk
Then the indicator shows Error and the failure is visible in the UI

13. Clipboard fallback
Given type-at-cursor is unavailable on the current Wayland setup
When the user completes a dictation
Then the transcript is copied to the clipboard as a fallback
