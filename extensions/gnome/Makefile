UUID     := voxput@jonochang.github.com
EXTDIR   := $(HOME)/.local/share/gnome-shell/extensions/$(UUID)
SCHEMADIR := $(EXTDIR)/schemas
SRCDIR   := $(UUID)

.PHONY: install uninstall compile-schemas pack

install: compile-schemas
	mkdir -p "$(EXTDIR)"
	cp -r "$(SRCDIR)/." "$(EXTDIR)/"
	glib-compile-schemas "$(SCHEMADIR)"
	@echo "Installed to $(EXTDIR)"
	@echo "Restart GNOME Shell (Alt+F2 → 'r') then enable with:"
	@echo "  gnome-extensions enable $(UUID)"

uninstall:
	rm -rf "$(EXTDIR)"
	@echo "Removed $(EXTDIR)"

compile-schemas:
	glib-compile-schemas "$(SRCDIR)/schemas/"

pack: compile-schemas
	cd "$(SRCDIR)" && \
	zip -r "../$(UUID).zip" . --exclude "*.zip"
	@echo "Created $(UUID).zip"
