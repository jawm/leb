# leb
Rust crate for reading Length Encoded Binary numbers. Used in the JumpJet crate.

This crate provides a `ReadLEB` trait, which has methods `read_varint` and `read_varuint`. There is also an implementation of these methods for the `std::io::Bytes` type. These methods take a parameter specifying the maximum number of bits allowed for the number, which is useful when parsing WASM.

This crate was designed to be used in my [JumpJet](https://github.com/jawm/jumpjet) crate, but is generic and could be used elsewhere.

Contributions / critique welcome.
