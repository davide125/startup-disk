DESTDIR ?=
PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share

RUSTFLAGS ?= --release

ROOTDIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

all: build

appdata-test:
	gnome-software --show-metainfo=$(ROOTDIR)/res/org.gnome.StartupDisk.metainfo.xml,icon=$(ROOTDIR)/res/org.gnome.StartupDisk.svg

appdata-validate:
	appstream-util validate-strict res/org.gnome.StartupDisk.metainfo.xml

build:
	cargo build $(RUSTFLAGS)

check: check-bin check-data

check-bin:
	cargo test $(RUSTFLAGS)

check-data:
	desktop-file-validate res/org.gnome.StartupDisk.desktop
	appstream-util validate-relax --nonet res/org.gnome.StartupDisk.metainfo.xml

clean:
	rm -rf target

install: install-bin install-data update-caches

install-bin:
	install -Dpm0755 -t $(DESTDIR)$(BINDIR)/ target/release/startup-disk

install-data:
	desktop-file-install --dir=$(DESTDIR)$(DATADIR)/applications/ res/org.gnome.StartupDisk.desktop
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/ res/org.gnome.StartupDisk.svg
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/metainfo/ res/org.gnome.StartupDisk.metainfo.xml
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/polkit-1/actions/ res/org.gnome.StartupDisk.policy

uninstall: uninstall-bin uninstall-data update-caches

uninstall-bin:
	rm -f $(DESTDIR)$(BINDIR)/startup-disk

uninstall-data:
	rm -f $(DESTDIR)$(DATADIR)/applications/org.gnome.StartupDisk.desktop
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/org.gnome.StartupDisk.svg
	rm -f $(DESTDIR)$(DATADIR)/metainfo/org.gnome.StartupDisk.metainfo.xml
	rm -f $(DESTDIR)$(DATADIR)/polkit-1/actions/org.gnome.StartupDisk.policy

update-caches:
	gtk-update-icon-cache --force --ignore-theme-index $(DESTDIR)$(DATADIR)/icons/hicolor
	update-desktop-database $(DESTDIR)$(DATADIR)/applications

.PHONY: appdata-test appdata-validate check check-bin check-data install install-bin install-data uninstall uninstall-bin uninstall-data update-caches
