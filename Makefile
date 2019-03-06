
clean:
	cargo clean

osx: clean
	MACOSX_DEPLOYMENT_TARGET=10.10 cargo build -v --release --target x86_64-apple-darwin --frozen
	strip target/x86_64-apple-darwin/release/topiks
	file target/x86_64-apple-darwin/release/topiks

linux: clean
	cargo build -v --release --target x86_64-unknown-linux-musl --frozen
	strip target/x86_64-unknown-linux-musl/release/topiks
	file target/x86_64-unknown-linux-musl/release/topiks
