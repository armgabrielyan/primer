RECIPE_ID ?= operating-system
RECIPE_DIR := recipes/$(RECIPE_ID)

.PHONY: help test test-all validate lint-shell check

help:
	@echo "Available targets:"
	@echo "  make test                      Run all automated test suites"
	@echo "  make validate RECIPE_ID=<id>   Validate one recipe contract + structure"
	@echo "  make lint-shell RECIPE_ID=<id> Bash syntax check for milestone scripts"
	@echo "  make check RECIPE_ID=<id>      Run validate + test + lint-shell"

test:
	@tests/recipe-validation/run
	@tests/shared-commands/run
	@tests/claude-adapter/run
	@tests/codex-adapter/run

test-all: test

validate:
	@scripts/validate-recipe "$(RECIPE_DIR)"

lint-shell:
	@find "$(RECIPE_DIR)/milestones" -type f \( -path "*/tests/check.sh" -o -name "demo.sh" \) | while IFS= read -r f; do \
		bash -n "$$f"; \
	done

check: validate test lint-shell
