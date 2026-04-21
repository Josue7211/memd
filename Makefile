SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

.PHONY: help backlog-index backlog-lint lint-links roadmap-audit handoff-latest docs-green bench-public bench-public-memd bench-mempalace bench

help:
	@echo "memd repo targets:"
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
