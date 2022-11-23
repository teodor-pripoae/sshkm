.PHONY: build doc deb

all: build doc deb

build:
	cargo build --release

doc:
	mkdir -p doc
	help2man ./target/release/sshkm > doc/sshkm.8
	gzip -f -k doc/sshkm.8

deb:
	mkdir -p deb
	cargo deb --no-build
	mv target/debian/sshkm_*.deb deb/
	rm -rf target/debian