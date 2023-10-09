SRC := ./src

.PHONY: test
test:
	@cargo test --manifest-path $(SRC)/Cargo.toml --workspace

.PHONY: clean
clean:
	@rm -rf $(SRC)/Cargo.lock
	@rm -rf $(SRC)/target/
