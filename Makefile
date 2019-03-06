
clean:
	cargo clean

test: clean
	cargo -v test

topiks: test
	rustup target add $RUST_TARGET
	cargo build -v --release --target $RUST_TARGET
	strip target/$RUST_TARGET/release/topiks
	file target/$RUST_TARGET/release/topiks
