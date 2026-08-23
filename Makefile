APP_NAME = $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')
APP_VERSION = $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')
PRODUCT_NAME = $(shell jq -r '.productName' tauri.conf.json | sed 's/ /\\ /g')
BUNDLE_DIR = target/release/bundle
ARCH_DIR = $(BUNDLE_DIR)/arch/$(APP_NAME)-$(APP_VERSION)-1-x86_64
ARCH_BUNDLE = $(ARCH_DIR).pkg.tar.gz
DEB_BUNDLE = $(BUNDLE_DIR)/deb/$(PRODUCT_NAME)_$(APP_VERSION)_amd64.deb
RPM_BUNDLE = $(BUNDLE_DIR)/rpm/$(PRODUCT_NAME)-$(APP_VERSION)-1.x86_64.rpm
APPIMAGE_BUNDLE = $(BUNDLE_DIR)/appimage/$(PRODUCT_NAME)_$(APP_VERSION)_amd64.AppImage 

.PHONY: all
all: arch-bundle

$(DEB_BUNDLE) $(RPM_BUNDLE) $(APPIMAGE_BUNDLE):
	cargo tauri build

.PHONY: arch-bundle
arch-bundle: $(ARCH_BUNDLE)

$(ARCH_BUNDLE): $(DEB_BUNDLE)
	mkdir -p $(ARCH_DIR)
	cp packaging/arch/whatsapp-desktop.install $(ARCH_DIR)/
	cargo metadata --format-version=1 --no-deps | jq -r -f packaging/arch/PKGBUILD.jq > $(ARCH_DIR)/PKGBUILD
	cd $(ARCH_DIR) && PKGDEST=.. makepkg --nodeps -f

.PHONY: clean
clean:
	cargo clean
