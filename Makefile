
clean:
	cargo clean

test: clean
	cargo -v test

osx: test
	MACOSX_DEPLOYMENT_TARGET=10.10 cargo build -v --release --target x86_64-apple-darwin
	strip target/x86_64-apple-darwin/release/topiks
	file target/x86_64-apple-darwin/release/topiks

linux: test
	rustup target add x86_64-unknown-linux-musl
	cargo build -v --release --target x86_64-unknown-linux-musl
	strip target/x86_64-unknown-linux-musl/release/topiks
	file target/x86_64-unknown-linux-musl/release/topiks
