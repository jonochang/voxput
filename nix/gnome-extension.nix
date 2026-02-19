{ stdenvNoCC, glib, lib }:

stdenvNoCC.mkDerivation {
  pname = "gnome-shell-extension-voxput";
  version = "0.3.0";

  # The '@' in the directory name is illegal in Nix store paths.
  # builtins.path lets us supply a clean 'name' while still reading the real dir.
  src = builtins.path {
    name = "voxput-gnome-extension-src";
    path = toString ../extensions/gnome + "/voxput@jonochang.github.com";
  };

  nativeBuildInputs = [ glib ];

  installPhase = ''
    runHook preInstall

    local extdir="$out/share/gnome-shell/extensions/voxput@jonochang.github.com"
    local schemadir="$out/share/glib-2.0/schemas"

    mkdir -p "$extdir" "$schemadir"

    # Copy all extension files
    cp -r . "$extdir/"

    # Install schemas to the standard NixOS location so GNOME can find them
    cp schemas/*.xml "$schemadir/"
    glib-compile-schemas "$schemadir"

    runHook postInstall
  '';

  meta = {
    description = "GNOME Shell extension for voxput voice dictation";
    homepage = "https://github.com/jonochang/voxput";
    license = lib.licenses.mit;
    # Requires GNOME Shell 45+
    platforms = lib.platforms.linux;
  };
}
