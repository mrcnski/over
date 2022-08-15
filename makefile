run: format clippy test build
	cargo run

build:
	cargo build

format:
	cargo fmt

test:
	cargo test

check:
	cargo check

clippy:
	cargo clippy --profile=test

doc:
	cargo doc

# Display module structure
modules:
	cargo modules --orphans tree

# Run fuzz
fuzz: fuzz1 fuzz2
fuzz1:
	cargo fuzz run fuzz_target_1 -- -max_len=4128 -max_total_time=600
fuzz2:
	cargo fuzz run fuzz_target_write -- -max_len=4128 -max_total_time=600
