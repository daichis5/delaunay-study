CARGO ?= cargo

POINTS_2D ?= data/square_center_2d.csv
ELEMENTS_2D ?= outputs/rust_2d.csv

.PHONY: setup format lint test run-2d clean help

help:
	@echo "setup    - fetch Rust dependencies"
	@echo "format   - apply rustfmt"
	@echo "lint     - check formatting and run clippy as errors"
	@echo "test     - run the Rust test suite"
	@echo "run-2d   - triangulate $(POINTS_2D) into $(ELEMENTS_2D)"
	@echo "clean    - remove generated files from outputs/"

setup:
	$(CARGO) fetch

format:
	$(CARGO) fmt

lint:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	$(CARGO) test

run-2d:
	$(CARGO) run --bin delaunay -- \
		--dimension 2 \
		--input $(POINTS_2D) \
		--output $(ELEMENTS_2D)

# Deliberately limited to the generated files under outputs/. Build artifacts are
# cargo's to manage, so use `cargo clean` for those; nothing here touches target/,
# data/, or the source tree.
clean:
	rm -f outputs/*.csv
