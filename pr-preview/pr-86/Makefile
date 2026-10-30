# Define our non-file based targets.
.PHONY: tui

# The terminal view, Rust built to WebAssembly in _site/tui. Needs the
# wasm32-unknown-unknown target and the wasm-bindgen CLI at the version in
# Cargo.lock. `cargo site build` keeps the folder when it rebuilds.
tui:
	cargo build --release --locked -p nixpulvis-web --target wasm32-unknown-unknown
	wasm-bindgen --target web --no-typescript --out-dir _site/tui --out-name tui \
		target/wasm32-unknown-unknown/release/nixpulvis_web.wasm
