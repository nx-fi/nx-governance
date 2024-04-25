# Makefile

WASM_PATH=target/wasm32-unknown-unknown/release

# Use Cargo.toml workspace.members as the source of truth for the projects to build
PROJ!=grep -o 'rs/[^"]*' Cargo.toml | sed 's/rs\///g'
PROJ_P_PREFIX!=grep -o 'rs/[^"]*' Cargo.toml | sed 's/rs\///g' | sed 's/^/-p /' | tr '\n' ' '

.PHONY: all
all: build did

.PHONY: install-toolchain
install-toolchain:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
	cargo install candid-extractor

.PHONY: setup
setup:
	cargo install candid-extractor

.PHONY: update
update:
	cargo update

.PHONY: clean
clean:
	cargo clean

.PHONY: test
test:
	cargo test

.PHONY: release
release:
	cargo build --target wasm32-unknown-unknown --release $(PROJ_P_PREFIX) --locked

.PHONY: did
did: release
	@echo $(PROJ) | tr ' ' '\n' | xargs -I {} sh -c "candid-extractor $(WASM_PATH)/{}.wasm >$(WASM_PATH)/{}.extracted.did"
	@echo "DID files extracted:"
	@find $(WASM_PATH) -name "*.extracted.did" -mmin -1

.PHONY: deploy
deploy: release
	sh ./deploy.sh