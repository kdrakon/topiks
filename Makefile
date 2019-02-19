
clean:
	cargo clean

# assumes local toolchain is on OSX
osx: clean
	cargo build -v --release --target x86_64-apple-darwin --frozen
	strip target/x86_64-apple-darwin/release/topiks
	file target/x86_64-apple-darwin/release/topiks

linux: clean
	docker run -it --rm -v $(PWD):$(PWD) -w $(PWD) rust:1.32.0 sh -c \
	'cargo build -v --release --target x86_64-unknown-linux-gnu \
		&& strip target/x86_64-unknown-linux-gnu/release/topiks \
		&& file target/x86_64-unknown-linux-gnu/release/topiks'
