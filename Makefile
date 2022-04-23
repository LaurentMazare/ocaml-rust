.PHONY: test
test:
	cargo build --manifest-path ./example/Cargo.toml
	cargo run --manifest-path ./gen/cmd/Cargo.toml -- --rust-file ./example/src/lib.rs --ocaml-file ./tests/ocaml/test_gen.ml
	cp ./target/debug/libocaml_rust_example.a tests/ocaml/
	dune runtest --root=tests/ocaml --force --no-buffer
	cargo test

promote:
	dune promote --root=tests/ocaml

test-exe:
	dune exec --root=tests/ocaml ./test_cmd.exe

clippy:
	cargo clippy --manifest-path ./macro/Cargo.toml
	cargo clippy --manifest-path ./gen/cmd/Cargo.toml
	cargo clippy --manifest-path ./example/Cargo.toml
	cargo clippy --manifest-path ./example-arrow/Cargo.toml
	cargo clippy

arrow-exe:
	cargo build --manifest-path ./example-arrow/Cargo.toml
	cargo run --manifest-path ./gen/cmd/Cargo.toml -- --rust-file ./example-arrow/src/lib.rs --ocaml-file ./tests/ocaml/arrow_gen.ml
	cp ./target/debug/libocaml_rust_arrow.a tests/ocaml/
	dune exec --root=tests/ocaml ./arrow_cmd.exe


