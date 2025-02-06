.PHONY: build
build:
	cargo build --release

.PHONY: check
check:
	cargo check
	$(MAKE) test
	cargo clippy
	$(MAKE) clippy_float_cast
	cargo fmt --check

.PHONY: test
test:
	cargo test --lib

.PHONY: clippy_float_cast
clippy_float_cast:
	cargo clippy -- \
		-W clippy::cast-possible-truncation \
		-W clippy::cast-sign-loss

.PHONY: clippy_nursery
clippy_nursery:
	cargo clippy -- -W clippy::nursery

.PHONY: clippy_cargo
clippy_cargo:
	cargo clippy -- -W clippy::cargo

.PHONY: clippy_pedantic
clippy_pedantic:
	cargo clippy -- \
		-W clippy::pedantic \
		-A clippy::single_match_else \
		-A clippy::uninlined-format-args \
		-A clippy::missing_errors_doc

# XXX Coverage recipes assume llvm-cov is installed:
.PHONY: coverage
coverage:
	cargo llvm-cov --lib --ignore-filename-regex 'tests\.rs'

.PHONY: coverage_html
coverage_html:
	cargo llvm-cov --lib --ignore-filename-regex 'tests\.rs' --open

.PHONY: install
install:
	cargo install --path .

.PHONY: tag
tag:
	git tag v$$(cargo pkgid | awk -F'#' '{print $$2}')
