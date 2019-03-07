SHA_COMMAND ?= shasum -a 512

clean:
	cargo clean

test: clean
	cargo test

build:
	rustup target add ${RUST_TARGET}
	cargo build -v --release --target ${RUST_TARGET}
	strip target/${RUST_TARGET}/release/topiks
	file target/${RUST_TARGET}/release/topiks

package: build
	cd target/${RUST_TARGET}/release/ && ${SHA_COMMAND} topiks > checksum-sha512
	tar vczf topiks-${RUST_TARGET}.tar.gz -C target/${RUST_TARGET}/release topiks checksum-sha512

linux-package:
	docker build --target ${LINUX_ENV} -t ${LINUX_ENV} -f Dockerfile-CI-Linux .
	docker run --rm -v ${PWD}:/topiks -e RUST_TARGET=${RUST_TARGET} ${LINUX_ENV}