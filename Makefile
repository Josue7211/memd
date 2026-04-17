SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

.PHONY: help backlog-index backlog-lint lint-links roadmap-audit handoff-latest docs-green

help:
	@echo "memd repo targets:"
	@echo "  backlog-index   - regenerate docs/backlog/INDEX.md from YAML frontmatter"
	@echo "  backlog-lint    - fail if backlog files lack phase: or INDEX.md is stale"
	@echo "  lint-links      - resolve every [[wikilink]] in docs/**; fail on broken"
	@echo "  roadmap-audit   - fail if any open backlog item has no live phase assigned"
	@echo "  handoff-latest  - refresh docs/handoff/LATEST.md symlink + INDEX.md"
	@echo "  docs-green      - run all of the above; exit non-zero on any failure"

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

docs-green: backlog-lint lint-links roadmap-audit
	@echo "docs-green: all gates passed"
