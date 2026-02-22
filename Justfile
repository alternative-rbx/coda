set shell := ["cmd.exe", "/c"]

publish:
    cargo publish --manifest-path crates/runtime/Cargo.toml
    cargo publish --manifest-path crates/std/Cargo.toml