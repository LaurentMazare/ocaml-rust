.PHONY: test
test:
	cargo build --manifest-path ./example/Cargo.toml
	cargo run --manifest-path ./gen/cmd/Cargo.toml -- --rust-file ./example/src/lib.rs --ocaml-file ./tests/basic/test_gen.ml
	cp ./target/debug/libocaml_rust_example.a tests/basic/
	dune runtest --root=tests/basic --force --no-buffer
	cargo run --manifest-path ./gen/cmd/Cargo.toml -- --rust-file ./example-arrow/src/lib.rs --ocaml-file ./tests/arrow/arrow_gen.ml
	cp ./target/debug/libocaml_rust_arrow.a tests/arrow/
	dune runtest --root=tests/arrow --force --no-buffer
	cargo test

promote:
	dune promote --root=tests/basic
	dune promote --root=tests/arrow

test-exe:
	dune exec --root=tests/basic ./test_cmd.exe

clippy:
	cargo clippy --manifest-path ./macro/Cargo.toml
	cargo clippy --manifest-path ./gen/cmd/Cargo.toml
	cargo clippy --manifest-path ./example/Cargo.toml
	cargo clippy --manifest-path ./example-arrow/Cargo.toml
	cargo clippy

arrow-exe:
	cargo build --manifest-path ./example-arrow/Cargo.toml
	cargo run --manifest-path ./gen/cmd/Cargo.toml -- --rust-file ./example-arrow/src/lib.rs --ocaml-file ./tests/arrow/arrow_gen.ml
	cp ./target/debug/libocaml_rust_arrow.a tests/arrow/
	dune exec --root=tests/arrow ./arrow_cmd.exe


