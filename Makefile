.PHONY: all build package extensions install-deb run clean help

VERSION ?= 3.0.3
DEB_PACKAGE := dist/aurora-kaushal-idm_$(VERSION)_amd64.deb

help:
	@echo "AURORA Kaushal IDM - Build & Installation Targets:"
	@echo "  make install-deb    - Reinstall the current .deb package on your system"
	@echo "  make build          - Build the React 19 UI & Rust/Tauri desktop binaries"
	@echo "  make package        - Package .deb, .tar.gz, Windows setup.exe, and macOS zip"
	@echo "  make extensions     - Build WASM and package extensions for Chrome, Firefox, Brave, Edge, Safari"
	@echo "  make run            - Launch the installed AURORA IDM application"
	@echo "  make clean          - Clean build cache and temporary artifacts"

install-deb:
	@echo "⚡ Installing AURORA Kaushal IDM v$(VERSION) (.deb)..."
	sudo apt install --reinstall -y ./$(DEB_PACKAGE) || sudo dpkg -i $(DEB_PACKAGE)
	@echo "✅ Reinstallation complete! You can run AURORA IDM from your app launcher or by typing: aurora-desktop"

build:
	@echo "⚡ Building Desktop Frontend UI & Rust Engine..."
	cd apps/aurora-desktop && npm run build
	cargo build --release -p aurora-desktop

package:
	@echo "⚡ Packaging all Multi-Platform Releases..."
	bash scripts/package_desktop.sh

extensions:
	@echo "⚡ Building Pure WASM & Dual Browser Extensions..."
	bash scripts/build_extension.sh

run:
	aurora-desktop &

clean:
	cargo clean --doc
	rm -rf target/debug target/tmp
