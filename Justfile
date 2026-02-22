set shell := ["cmd.exe", "/c"]

publish-runtime:
    cargo publish --manifest-path crates/runtime/Cargo.toml
    
publish-std:
    cargo publish --manifest-path crates/std/Cargo.toml