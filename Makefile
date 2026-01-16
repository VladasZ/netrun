
lint:
	cargo clippy \
      -- \
      \
      -W clippy::all \
      -W clippy::pedantic \
      \
      -A clippy::module_inception \
      -A clippy::missing_errors_doc \
      -A clippy::missing_panics_doc \
      -A clippy::needless_pass_by_value \
      -A clippy::must_use_candidate \
      -A clippy::manual_assert \
      -A clippy::return_self_not_must_use \
      \
      -D warnings


test:
	cargo test --all
	cargo test --all --release
	make test-wasm

test-wasm:
	cargo install wasm-pack
	cd netrun && wasm-pack test --firefox --headless
