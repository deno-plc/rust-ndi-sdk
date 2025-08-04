# rust-ndi-sdk (`ndi-sdk-sys`)

[![Crate](https://img.shields.io/crates/v/ndi-sdk-sys.svg)](https://crates.io/crates/ndi-sdk-sys)
[![Documentation](https://docs.rs/ndi-sdk-sys/badge.svg)](https://docs.rs/ndi-sdk-sys)

(WIP) Safe Rust bindings for the [NDI (ndi.video)](https://ndi.video/) SDK,
enabling high-performance video, audio, and metadata transmission over IP
networks.

## Features

- Safe wrapper around the NDI SDK
- Currently supports:
  - Finder API for discovering NDI sources on the network
  - Router API for routing NDI streams
  - Sender API for transmitting NDI streams
  - Receiver API for receiving NDI streams
- Not supported yet:
  - PTZ Control
  - FrameSync
  - Receiver advertisement

## Version compatibility

This crate is currently tested against SDK version 6.2.0.3

The raw bindings are generated from the header files in the SDK installation
directory, this way generated code will always match the version on the machine
it is compiled with and linked against.

## Platform Support

This crates `build.rs` is currently lacking support for platforms other than
Windows x64, because I had no time for it so far. If you are working on another
platform, feel free to add support for it. (It shouldn't be that complicated,
windows support is less than 30 lines of nearly-boilerplate code)

## Installation

1. Make sure to have the [NDI SDK](https://ndi.video/for-developers/#ndi-sdk)
   installed
2. Make sure to have Clang/LLVM installed.
   [see rust-bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)
3. Add `ndi-sdk-sys` to your cargo dependencies

Check out the examples at
https://github.com/deno-plc/rust-ndi-sdk/tree/main/examples

## Cargo features

### `strict_assertions`

This crate uses a lot of unsafe ffi bindings and a lot of assertions. Sometimes
we does not even trust the documentation and double check everything. For
example every resolution is checked for overflow when calculating the number of
pixels. (e.g. height and width are < usize::max, but their product may be
larger)

You can turn on this feature to ensure all these assertions are in place even
for release builds (for debug builds they are always on). During development and
testing it is highly recommended to turn this on, as it also outputs more
diagnostics.

## Safety

This crate provides safe abstractions of the NDI SDK. The public API is designed
to be 100% safe to use. If you encounter restrictive lifetimes, do not try to
circumvent them, most likely it just represents the safety requirements of the C
SDK.

As expected form a C library, safety requirements are not always documented
in-depth by the NDI SDK. One common assumption this crate uses it that it is
safe to drop arguments that are passed as pointers (and strings inside them)
immediately after the call they were passed to has returned (unless defined
otherwise by the documentation).

## Contributing

Contributions are welcome! Here's how you can help:

1. Add support for other platforms in `build.rs`
2. Report bugs and find missing APIs
3. Improve documentation and examples

## License

Copyright (C) 2025 Hans Schallmoser

Licensed under GPL-v3.

Note: The NDI SDK is subject to its own
[EULA](https://ndi.video/for-developers/#ndi-sdk).

## Disclaimer

This project is not affiliated with NDI® or NewTek/Vizrt
