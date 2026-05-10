SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

# Override on the command line, e.g.
#   make install-memd MEMD_INSTALL_PATH=$$HOME/.cargo/bin/memd
MEMD_INSTALL_PATH ?= /Volumes/T7/node/bin/memd
CARGO_TARGET_DIR  ?= /Volumes/T7/cargo-target

.PHONY: help backlog-index backlog-lint lint-links roadmap-audit handoff-latest docs-green bench-public bench-public-memd bench-mempalace bench build-memd install-memd

help:
	@echo "memd repo targets:"
	@echo "  build-memd        - cargo build --release -p memd-client"
	@echo "  install-memd      - build + copy + ad-hoc codesign into \$$MEMD_INSTALL_PATH"
	@echo "                      (codesign required on macOS 15+/Tahoe: fresh cargo binaries exit 137)"
	@echo "  backlog-index     - regenerate docs/backlog/INDEX.md from YAML frontmatter"
	@echo "  backlog-lint      - fail if backlog files lack phase: or INDEX.md is stale"
	@echo "  lint-links        - resolve every [[wikilink]] in docs/**; fail on broken"
	@echo "  roadmap-audit     - fail if any open backlog item has no live phase assigned"
	@echo "  handoff-latest    - refresh docs/handoff/LATEST.md symlink + INDEX.md"
	@echo "  bench-public      - rerun public benchmarks on lexical backend (M0-parity cadence)"
	@echo "  bench-public-memd - rerun public benchmarks against a running memd-server (G3 parity path)"
	@echo "  bench-mempalace   - replay MemPalace on memd benchmark fixtures and refresh replay artifacts"
	@echo "  bench             - alias for bench-public"
	@echo "  docs-green        - run all of the above; exit non-zero on any failure"

build-memd:
	CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo build --release -p memd-client --bin memd

install-memd: build-memd
	@if [ ! -x "$(CARGO_TARGET_DIR)/release/memd" ]; then \
		echo "build-memd did not produce $(CARGO_TARGET_DIR)/release/memd"; \
		exit 1; \
	fi
	@mkdir -p "$$(dirname $(MEMD_INSTALL_PATH))"
	@if [ -f "$(MEMD_INSTALL_PATH)" ]; then \
		backup="$(MEMD_INSTALL_PATH).backup-$$(date +%Y%m%d-%H%M%S)"; \
		echo "backing up existing binary -> $$backup"; \
		cp "$(MEMD_INSTALL_PATH)" "$$backup"; \
	fi
	cp "$(CARGO_TARGET_DIR)/release/memd" "$(MEMD_INSTALL_PATH)"
	@case "$$(uname -s)" in \
		Darwin) \
			echo "ad-hoc codesigning $(MEMD_INSTALL_PATH) (required on macOS 15+/Tahoe)"; \
			xattr -cr "$(MEMD_INSTALL_PATH)" 2>/dev/null || true; \
			codesign --force --sign - "$(MEMD_INSTALL_PATH)";; \
		*) echo "skipping codesign on $$(uname -s)";; \
	esac
	@"$(MEMD_INSTALL_PATH)" --help 2>&1 | head -1 || (echo "self-test failed"; exit 1)
	@echo "installed: $(MEMD_INSTALL_PATH)"

backlog-index:
	@bash scripts/backlog-index.sh

backlog-lint:
	@bash scripts/backlog-lint.sh

lint-links:
	@bash scripts/lint-links.sh

roadmap-audit:
	@bash scripts/roadmap-audit.sh

handoff-latest:
	@bash scripts/handoff-latest.sh

bench-public:
	cargo run -p memd-client -- benchmark public --all --write --record --out .memd

# G3: route every bench (LongMemEval, LoCoMo, MemBench, ConvoMem) through
# memd-server's intrinsic retrieval path. Requires a running memd-server;
# override MEMD_BASE_URL to point at a non-default host:port.
bench-public-memd:
	cargo run -p memd-client -- benchmark public --all --write --record --out .memd --retrieval-backend memd

bench-mempalace:
	/home/josue/Documents/projects/mempalace/.venv/bin/python scripts/bench-mempalace.py

bench: bench-public

docs-green: backlog-lint lint-links roadmap-audit
	@echo "docs-green: all gates passed"
