check:
	cargo audit
	cargo test
	cargo tarpaulin --ignore-tests # code coverage
	cargo clippy -- -D warnings # fail if there are warnings
	cargo fmt -- --check # fail if not properly formatted

