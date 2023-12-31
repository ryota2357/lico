.PHONY: help
help:
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@echo '  install  Install release version of the binary'
	@echo '  check    Check with `cargo check` and `cargo clippy`'
	@echo '  clean    Remove Cargo.lock and target/'
	@echo '  format   Format rust code'
	@echo '  help     Show this help'

.PHONY: install
install:
	@cargo install --path ./cli --force --jobs 4

.PHONY: format
format:
	@cargo fmt --manifest-path ./src/Cargo.toml --all --check
	@cargo fmt --manifest-path ./cli/Cargo.toml --all --check

.PHONY: check
check:
	@cargo check --manifest-path ./src/Cargo.toml --workspace
	@cargo check --manifest-path ./cli/Cargo.toml --workspace
	@cargo clippy --manifest-path ./src/Cargo.toml --workspace
	@cargo clippy --manifest-path ./cli/Cargo.toml --workspace

.PHONY: clean
clean:
	@rm -rf ./src/Cargo.lock
	@rm -rf ./src/target/
	@rm -rf ./cli/Cargo.lock
	@rm -rf ./cli/target/
