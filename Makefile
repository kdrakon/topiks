SHA_COMMAND ?= shasum -a 512
RUST_TARGET ?= x86_64-apple-darwin
OS_TARGET ?= osx-10.13

clean:
	cargo clean

test: clean
	cargo test

build: clean
	rustup target add ${RUST_TARGET}
	cargo build -v --release --target ${RUST_TARGET}
	strip target/${RUST_TARGET}/release/topiks
	file target/${RUST_TARGET}/release/topiks

package: build
	cd target/${RUST_TARGET}/release/ && ${SHA_COMMAND} topiks > checksum-sha512
	tar vczf topiks_${RUST_TARGET}_${OS_TARGET}.tar.gz -C target/${RUST_TARGET}/release topiks checksum-sha512

linux-package:
	docker build --target ${OS_TARGET} -t ${OS_TARGET} -f Dockerfile-CI-Linux .
	docker run --rm -v ${PWD}:/topiks -e RUST_TARGET=${RUST_TARGET} -e OS_TARGET=${OS_TARGET} ${OS_TARGET}
