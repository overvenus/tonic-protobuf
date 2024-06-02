# tonic-build-protobuf / tonic-codec-protobuf

[![Crates.io](https://img.shields.io/crates/v/tonic-build-protobuf)](https://crates.io/crates/tonic-build-protobuf)
[![Documentation](https://docs.rs/tonic-build-protobuf/badge.svg)](https://docs.rs/tonic-build-protobuf)
[![LICENSE](https://img.shields.io/crates/l/tonic-build-protobuf)](LICENSE)

Compiles proto files via [rust-protobuf](https://crates.io/crates/protobuf) and generates service stubs for use with tonic.

## Features

Required dependencies

```toml
[dependencies]
tonic = "<tonic-version>"
protobuf = "<protobuf-version>"
tonic-codec-protobuf = "<tonic-codec-protobuf-version>"

[build-dependencies]
tonic-build-protobuf = "<tonic-build-protobuf-version>"
```

## Examples

In `build.rs`:

```rust,ignore
fn main() {
    // Project layout:
    // .
    // ├── Cargo.toml
    // ├── build.rs
    // ├── include
    // │   └── rustproto.proto
    // ├── proto
    // │   └── debugpb.proto
    // └── src
    //     └── lib.rs
    tonic_build_protobuf::Builder::new()
        .out_dir(format!(
            "{}/protos",
            std::env::var("OUT_DIR").expect("No OUT_DIR defined")
        ))
        .proto_path("crate")
        .file_name(|pkg, svc| format!("{pkg}_{svc}_tonic"))
        .codec_path("::tonic_codec_protobuf::ProtobufCodecV3")
        .compile(&["proto/debugpb.proto"], &["proto", "include"]);
}
```

Then you can reference the generated Rust like this this in your code:

```rust,ignore
mod generated {
    include!(concat!(env!("OUT_DIR"), "debugpb_debug_tonic.rs"));
}

pub use generated::*;
```

See [examples here](https://github.com/overvenus/tonic-protobuf/tree/master/examples)

## License

This project is licensed under the [MIT license](https://github.com/overvenus/tonic-protobuf/blob/main/LICENSE).
