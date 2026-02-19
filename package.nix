# This file goes in nixpkgs at: pkgs/by-name/vo/voxput/package.nix
#
# To get the real cargoHash, set it to lib.fakeHash, run `nix build`,
# and the error output will contain the correct hash.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  alsa-lib,
  testers,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "voxput";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "jonochang";
    repo = "voxput";
    tag = "v${finalAttrs.version}";
    hash = lib.fakeHash;
  };

  cargoHash = lib.fakeHash;

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
