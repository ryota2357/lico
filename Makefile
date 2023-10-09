SRC := ./src

.PHONY: format
format:
	@cargo fmt --manifest-path $(SRC)/Cargo.toml --all --check

.PHONY: check
check:
	@cargo check --manifest-path $(SRC)/Cargo.toml --workspace
	@cargo clippy --manifest-path $(SRC)/Cargo.toml --workspace

.PHONY: test
test:
	@cargo test --manifest-path $(SRC)/Cargo.toml --workspace

.PHONY: clean
clean:
	@rm -rf $(SRC)/Cargo.lock
	@rm -rf $(SRC)/target/
