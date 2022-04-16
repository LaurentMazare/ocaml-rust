.PHONY: test
test:
	cargo build --manifest-path ./example/Cargo.toml
	cargo run --manifest-path ./gen/cmd/Cargo.toml -- --rust-file ./example/src/lib.rs --ocaml-file ./tests/ocaml/test_gen.ml
	cp ./target/debug/libocaml_rust_example.a tests/ocaml/
	dune runtest --root=tests/ocaml --force --no-buffer

promote:
	dune promote --root=tests/ocaml
