GOGS_REPO    := sarman/tftsr-devops_investigation
TAG        ?= v0.1.0-alpha
TARGET     := aarch64-unknown-linux-gnu

# Build linux/arm64 release artifact natively inside a Docker container,
# then upload to the GitHub release for TAG.
.PHONY: release-arm64
release-arm64: build-arm64 upload-arm64

.PHONY: build-arm64
build-arm64:
	docker run --rm \
		--platform linux/arm64 \
		-v "$(CURDIR):/workspace" \
		-w /workspace \
		rust:1.88-slim \
		bash -c ' \
			apt-get update -qq && \
			apt-get install -y -qq libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
				libayatana-appindicator3-dev librsvg2-dev patchelf pkg-config curl perl && \
			curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
			apt-get install -y nodejs && \
			npm ci --legacy-peer-deps && \
			rustup target add $(TARGET) && \
			cargo install tauri-cli --version "^2" --locked && \
			CI=true cargo tauri build --target $(TARGET) && \
			mkdir -p artifacts/linux-arm64 && \
			find src-tauri/target/$(TARGET)/release/bundle \
				-name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" | \
				xargs -I{} cp {} artifacts/linux-arm64/ \
		'
	@echo "Artifacts:"
	@ls -lh artifacts/linux-arm64/

.PHONY: upload-arm64
upload-arm64:
	@test -n "$(GOGS_TOKEN)" || (echo "ERROR: set GOGS_TOKEN env var"; exit 1)
	@for f in artifacts/linux-arm64/*; do \
		[ -f "$$f" ] || continue; \
		NAME="linux-arm64-$$(basename $$f)"; \
		echo "Uploading $$NAME..."; \
		GOGS_TOKEN=$(GOGS_TOKEN) # gh release upload $(TAG) "$$f#$$NAME" \
			--repo $(GOGS_REPO) && echo "OK" || echo "FAIL: $$f"; \
	done
