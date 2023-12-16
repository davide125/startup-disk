DESTDIR ?=
PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share

RUSTFLAGS ?= --release

ROOTDIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
APP_ID := org.startup_disk.StartupDisk

all: build

appdata-test:
	gnome-software --show-metainfo=$(ROOTDIR)/res/$(APP_ID).metainfo.xml,icon=$(ROOTDIR)/res/$(APP_ID).svg

appdata-validate:
	appstream-util validate-strict res/$(APP_ID).metainfo.xml

build:
	cargo build $(RUSTFLAGS)

check: check-bin check-data

check-bin:
	cargo test $(RUSTFLAGS)

check-data:
	desktop-file-validate res/$(APP_ID).desktop
	appstream-util validate-relax --nonet res/$(APP_ID).metainfo.xml

clean:
	rm -rf target

install: install-bin install-data update-caches

install-bin:
	install -Dpm0755 -t $(DESTDIR)$(BINDIR)/ target/release/startup-disk

install-data:
	desktop-file-install --dir=$(DESTDIR)$(DATADIR)/applications/ res/$(APP_ID).desktop
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/ res/$(APP_ID).svg
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/metainfo/ res/$(APP_ID).metainfo.xml
	install -Dpm0644 -t $(DESTDIR)$(DATADIR)/polkit-1/actions/ res/$(APP_ID).policy

uninstall: uninstall-bin uninstall-data update-caches

uninstall-bin:
	rm -f $(DESTDIR)$(BINDIR)/startup-disk

uninstall-data:
	rm -f $(DESTDIR)$(DATADIR)/applications/$(APP_ID).desktop
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg
	rm -f $(DESTDIR)$(DATADIR)/metainfo/$(APP_ID).metainfo.xml
	rm -f $(DESTDIR)$(DATADIR)/polkit-1/actions/$(APP_ID).policy

update-caches:
	gtk-update-icon-cache --force --ignore-theme-index $(DESTDIR)$(DATADIR)/icons/hicolor
	update-desktop-database $(DESTDIR)$(DATADIR)/applications

.PHONY: appdata-test appdata-validate check check-bin check-data install install-bin install-data uninstall uninstall-bin uninstall-data update-caches
