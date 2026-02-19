UUID      := voxput@jonochang.github.com
EXTDIR    := $(HOME)/.local/share/gnome-shell/extensions/$(UUID)
SCHEMADIR := $(EXTDIR)/schemas
DBUSDIR   := $(HOME)/.local/share/dbus-1/services
SRCDIR    := $(UUID)

# Path to the D-Bus session activation file (two levels up from here)
DBUS_SERVICE_SRC := ../../contrib/com.github.jonochang.Voxput.service

.PHONY: install uninstall compile-schemas pack

install: compile-schemas
	# Extension files
	mkdir -p "$(EXTDIR)"
	cp -r "$(SRCDIR)/." "$(EXTDIR)/"
	glib-compile-schemas "$(SCHEMADIR)"
	# D-Bus session activation so voxputd auto-starts on first use
	mkdir -p "$(DBUSDIR)"
	cp "$(DBUS_SERVICE_SRC)" "$(DBUSDIR)/com.github.jonochang.Voxput.service"
	@echo ""
	@echo "Installed extension to:    $(EXTDIR)"
	@echo "Installed D-Bus service to: $(DBUSDIR)"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Make sure voxputd is on your PATH (~/.local/bin/voxputd)"
	@echo "  2. Set GROQ_API_KEY:  systemctl --user set-environment GROQ_API_KEY=gsk_..."
	@echo "  3. Restart GNOME Shell:"
	@echo "       Wayland: log out and back in"
	@echo "       X11:     Alt+F2 → type 'r' → Enter"
	@echo "  4. Enable the extension:"
	@echo "       gnome-extensions enable $(UUID)"
	@echo "     or open the Extensions app and toggle Voxput on"

uninstall:
	rm -rf "$(EXTDIR)"
	rm -f "$(DBUSDIR)/com.github.jonochang.Voxput.service"
	@echo "Removed extension and D-Bus service file."

compile-schemas:
	glib-compile-schemas "$(SRCDIR)/schemas/"

pack: compile-schemas
	cd "$(SRCDIR)" && \
	zip -r "../$(UUID).zip" . --exclude "*.zip"
	@echo "Created $(UUID).zip"
