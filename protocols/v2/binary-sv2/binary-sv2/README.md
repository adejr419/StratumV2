# binary-sv2

[![crates.io](https://img.shields.io/crates/v/binary-sv2.svg)](https://crates.io/crates/binary-sv2)
[![docs.rs](https://docs.rs/binary-sv2/badge.svg)](https://docs.rs/binary-sv2)
[![rustc+](https://img.shields.io/badge/rustc-1.75.0%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/stratum-mining/stratum/blob/main/LICENSE.md)

`binary-sv2` is a Rust crate that mediates between two implementations of Sv2 (Stratum V2) protocol serialization and deserialization—either through `serde` or custom trait-based encoding. This flexibility makes it adaptable for use in environments that may not support `serde`.

## Key Capabilities

- **Dual Serialization Options**: Offers both `serde` and custom serialization/deserialization, configurable with feature flags.
- **Protocol-Specific Types**: Supports fixed and dynamically-sized SV2 types.
- **Optimized Memory Use**: Supports buffer pooling to enhance memory efficiency.

## Features

- **with_serde**: Enables `serde`-based encoding and decoding.
- **core**: Activates non-`serde` implementations via `binary_codec_sv2` and `derive_codec_sv2`.
- **prop_test**: Adds property testing support.
- **with_buffer_pool**: Optimizes memory usage during serialization.

## Usage

To include this crate in your project, run:

```sh
cargo add binary-sv2
```


# binary_sv2

`binary_sv2` is a Rust `no_std` crate