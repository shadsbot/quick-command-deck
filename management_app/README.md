## Management App

\* name not final

The purpose of this application is to listen on a specified serial port for messages from the hardware, acting and responding accordingly. Primary communication is done by sending ProtoBuf messages through Serial. This may seem overkill, but as of writing this I don't quite know the full scope of this project, so having the option to expand in the future or make it easier to adapt for other systems/languages seems like a plus. 

### Libraries Used
Honestly just look at [Cargo.toml](Cargo.toml) for the full list of dependencies but here's what's going on behind the scenes.

- [susurrus/serialport](https://crates.io/crates/serialport) - This does a good bulk of the lifting; build on top of the serial library for Rust, serialport is responsible for sending and recieving messages. It is licensed under the [Mozilla Public License v2.0](https://www.mozilla.org/en-US/MPL/2.0/)
- [stepancheg/protobuf](https://github.com/stepancheg/rust-protobuf/) - Responsible for encoding and decoding protobuf messages. Also comes with a really nifty builder, `protobuf-codegen-pure`, which handles the `.proto` file used in this project. Licensed under the [MIT License](https://github.com/stepancheg/rust-protobuf/blob/master/LICENSE.txt).
- [pretty_env_logger](https://crates.io/crates/pretty_env_logger) - Responsible for log messages scattered throughout the program. Licensed under [MIT](https://github.com/seanmonstar/pretty-env-logger/blob/master/LICENSE-MIT) or [Apache v2](https://github.com/seanmonstar/pretty-env-logger/blob/master/LICENSE-APACHE).