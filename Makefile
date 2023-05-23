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
