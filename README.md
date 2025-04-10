# rust-ndi-sdk (`ndi-sdk-sys`)

[![Crate](https://img.shields.io/crates/v/ndi-sdk-sys.svg)](https://crates.io/crates/ndi-sdk-sys)
[![Documentation](https://docs.rs/ndi-sdk-sys/badge.svg)](https://docs.rs/ndi-sdk-sys)

(WIP) Safe Rust bindings for the [NDI](https://ndi.video/) SDK, enabling
high-performance video, audio, and metadata transmission over IP networks.

## Features

- Safe wrapper around the NDI SDK
- Currently supports:
  - Finder API for discovering NDI sources
  - Router API for routing NDI streams
- Not implemented yet:
  - Sender API for transmitting NDI streams
  - Receiver API for receiving NDI streams

## Platform Support

This crates `build.rs` is currently lacking support for platforms other than
Windows x64, if you are working on another platform, feel free to add support
for it.

## Installation

1. Make sure to have the [NDI SDK](https://ndi.video/for-developers/#ndi-sdk)
   installed
2. Add `ndi-sdk-sys` ro your cargo dependencies

Check out the examples at
https://github.com/deno-plc/rust-ndi-sdk/tree/main/examples

## Safety

This crate provides safe abstractions of the NDI SDK. The public API is designed
to be 100% safe to use. If you encounter restrictive lifetimes, do not try to
circumvent them, mot likely it just represents the safety requirements of the C
SDK.

## Contributing

Contributions are welcome! Here's how you can help:

1. Add support for other platforms in `build.rs`
2. Implement bindings for Receiver/Sender
3. Report bugs and suggest features
4. Improve documentation and examples

## License

Copyright (C) 2025 Hans Schallmoser

Licensed under GPL-v3.

Note: The NDI SDK is subject to its own
[EULA](https://ndi.video/for-developers/#ndi-sdk).

## Disclaimer

This project is not affiliated with NDI® or NewTek/Vizrt
