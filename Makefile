DESTDIR ?=
PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share

all: build

build:
	cargo build --release

install: install-bin install-data

install-bin:
	install -Dpm0755 -t $(DESTDIR)$(BINDIR)/ target/release/startup-disk

install-data:
	desktop-file-install --dir=$(DESTDIR)$(DATADIR)/applications/ res/org.gnome.StartupDisk.desktop
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/ res/org.gnome.StartupDisk.svg
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/metainfo/ res/org.gnome.StartupDisk.metainfo.xml
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/polkit-1/actions/ res/org.gnome.StartupDisk.policy

uninstall: uninstall-bin uninstall-data

uninstall-bin:
	rm -f $(DESTDIR)$(BINDIR)/startup-disk

uninstall-data:
	rm -f $(DESTDIR)$(DATADIR)/applications/org.gnome.StartupDisk.desktop
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/org.gnome.StartupDisk.svg
	rm -f $(DESTDIR)$(DATADIR)/metainfo/org.gnome.StartupDisk.metainfo.xml
	rm -f $(DESTDIR)$(DATADIR)/polkit-1/actions/org.gnome.StartupDisk.policy

.PHONY: install-bin install-data uninstall-bin uninstall-data
