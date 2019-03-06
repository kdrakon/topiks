
clean:
	cargo clean

test: clean
	cargo test

topiks: test
	rustup target add ${RUST_TARGET}
	cargo build -v --release --frozen --target ${RUST_TARGET}
	strip target/${RUST_TARGET}/release/topiks
	file target/${RUST_TARGET}/release/topiks

package:
	cd target/${RUST_TARGET}/release/ && shasum -a 512 topiks > checksum-sha512
	tar vczf topiks-${RUST_TARGET}.tar.gz -C target/${RUST_TARGET}/release topiks checksum-sha512
