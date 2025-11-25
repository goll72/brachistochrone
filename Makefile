WASM := target/wasm32-unknown-unknown/release/brachistochrone.wasm
SCRIPT := web/wasm/brachistochrone.js

all: $(SCRIPT)

serve: $(SCRIPT)
	python -m http.server -d web

$(WASM): $(wildcard src/*.rs) Cargo.toml Cargo.lock
	cargo build --release --target wasm32-unknown-unknown

$(SCRIPT): $(WASM)
	mv $(WASM) $(WASM).orig
	wasm-opt -Os -g $(WASM).orig -o $(WASM)
	wasm-bindgen --out-dir web/wasm --target web $(WASM)

.PHONY: all serve
