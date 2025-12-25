all: build

build:
	cargo build --release

check: lint test

clean:
	cargo clean

fmt: format

format:
	rustup component add --toolchain nightly-x86_64-unknown-linux-gnu rustfmt
	cargo +nightly fmt

lint:
	cargo clippy
	typos

purge: clean
	rm -rf .vat-cache

softrun: build
	@target/release/vat -p | tee vat.log
	@sed -i 's,\x1b\[[0-9;]*m,,g' vat.log

run: build
	@target/release/vat | tee vat.log
	@sed -i 's,\x1b\[[0-9;]*m,,g' vat.log
	@./commit.sh

test: build
	@cargo test --no-fail-fast --future-incompat-report --all-features --locked --release
	@target/release/vat -pg | tee vat.log
	@sed -i 's,\x1b\[[0-9;]*m,,g' vat.log
	@grep -E 'ERROR|WARN' vat.log || true
	@awk -v f=$$(cat .vat-cache/failed) -v c=$$(cat .vat-cache/checked) \
		'BEGIN { exit !(f/c < 0.05) }'

release:
	@./release.sh

.PHONY: all build check clean fmt format lint purge run test
