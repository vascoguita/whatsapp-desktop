APP_NAME = $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')
APP_VERSION = $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')
PRODUCT_NAME_RAW = $(shell jq -r '.productName' tauri.conf.json)
PRODUCT_NAME = $(shell jq -r '.productName' tauri.conf.json | sed 's/ /\\ /g')
PRODUCT_NAME_SAFE = $(shell jq -r '.productName' tauri.conf.json | tr ' ' '-')
BUNDLE_DIR = target/release/bundle
ARCH_DIR = $(BUNDLE_DIR)/arch/$(APP_NAME)-$(APP_VERSION)-1-x86_64
ARCH_BUNDLE = $(ARCH_DIR).pkg.tar.zst
DEB_BUNDLE = $(BUNDLE_DIR)/deb/$(PRODUCT_NAME)_$(APP_VERSION)_amd64.deb
# Gemfury's yum repodata mis-encodes spaces in rpm filenames (breaks `dnf install`),
# so this rpm is named without spaces, unlike the deb/AppImage bundles.
RPM_BUNDLE = $(BUNDLE_DIR)/rpm/$(PRODUCT_NAME_SAFE)-$(APP_VERSION)-1.x86_64.rpm
APPIMAGE_BUNDLE = $(BUNDLE_DIR)/appimage/$(PRODUCT_NAME)_$(APP_VERSION)_amd64.AppImage

.PHONY: all
all: arch-bundle

.PHONY: bundles
bundles: $(DEB_BUNDLE) $(RPM_BUNDLE) $(APPIMAGE_BUNDLE)

$(DEB_BUNDLE):
	cargo tauri build --bundles deb

$(RPM_BUNDLE):
	cargo tauri build --bundles rpm
	mv "$(BUNDLE_DIR)/rpm/$(PRODUCT_NAME_RAW)-$(APP_VERSION)-1.x86_64.rpm" "$@"

$(APPIMAGE_BUNDLE):
	cargo tauri build --bundles appimage

.PHONY: arch-bundle
arch-bundle: $(ARCH_BUNDLE)

$(ARCH_BUNDLE): $(DEB_BUNDLE)
	mkdir -p $(ARCH_DIR)
	cp packaging/arch/whatsapp-desktop.install $(ARCH_DIR)/
	cargo metadata --format-version=1 --no-deps | jq -r -f packaging/arch/PKGBUILD.jq > $(ARCH_DIR)/PKGBUILD
	cd $(ARCH_DIR) && PKGEXT='.pkg.tar.zst' PKGDEST=.. makepkg --nodeps -f

.PHONY: clean
clean:
	cargo clean
	rm -rf gen
