GOGS_API   := http://172.0.0.29:3000/api/v1
GOGS_REPO  := sarman/tftsr-devops_investigation
TAG        ?= v0.1.0-alpha
TARGET     := aarch64-unknown-linux-gnu

# Build linux/arm64 release artifact natively inside a Docker container,
# then upload to the Gogs release for TAG.
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
	@RELEASE_ID=$$(curl -sf "$(GOGS_API)/repos/$(GOGS_REPO)/releases/tags/$(TAG)" \
		-H "Authorization: token $(GOGS_TOKEN)" | \
		grep -o '"id":[0-9]*' | head -1 | cut -d: -f2); \
	echo "Release ID: $$RELEASE_ID"; \
	for f in artifacts/linux-arm64/*; do \
		[ -f "$$f" ] || continue; \
		echo "Uploading $$f..."; \
		curl -sf -X POST "$(GOGS_API)/repos/$(GOGS_REPO)/releases/$$RELEASE_ID/assets" \
			-H "Authorization: token $(GOGS_TOKEN)" \
			-F "attachment=@$$f;filename=$$(basename $$f)" && echo "OK" || echo "FAIL: $$f"; \
	done
