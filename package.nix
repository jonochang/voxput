# Single package definition used by both the flake overlay and nixpkgs.
#
# Flake overlay (src = self, no tag needed):
#   voxput = final.callPackage ./package.nix { src = self; };
#
# nixpkgs submission (src fetched by tag, fill in real hashes):
#   voxput = callPackage ./package.nix { };
#   → set hash in fetchFromGitHub and push a v<version> tag first.
{
  lib,
  src ? null,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  alsa-lib,
  testers,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "voxput";
  version = "0.3.0";

  src =
    if src != null
    then src
    else
      fetchFromGitHub {
        owner = "jonochang";
        repo = "voxput";
        tag = "v${finalAttrs.version}";
        hash = lib.fakeHash;
      };

  # Reads Cargo.lock from the source — works for both self and fetchFromGitHub
  # (the tarball includes Cargo.lock).  No cargoHash to maintain.
  cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    alsa-lib
  ];

  # Integration tests require a live microphone and GROQ_API_KEY
  checkFlags = [
    "--skip=groq_integration"
  ];

  passthru.tests.version = testers.testVersion {
    package = finalAttrs.finalPackage;
    command = "voxput --version";
  };

  meta = {
    description = "Voice-to-text dictation tool powered by Groq Whisper";
    homepage = "https://github.com/jonochang/voxput";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ jonochang ];
    mainProgram = "voxput";
    platforms = lib.platforms.linux;
  };
})
