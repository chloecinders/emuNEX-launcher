# emuNEX (launcher)
Remote Emulation & Library Management.

This launcher provides automatic installation and updates to the emuNEX server. Using this is highly recommended but not required.

Find the source code for the server here: https://github.com/chloecinders/emuNEX-server

## Building From Source

Make sure Git and Rust is installed, preferrably through [rustup](https://rust-lang.org/tools/install/).
Once Rust is installed run the following commands:
```bash
git clone https://github.com/chloecinders/emuNEX-launcher.git
cd emuNEX-launcher
cargo build --release
```
The binary will be built to `./target/{toolchain}/release/emunex-launcher(.exe)`.

## Contributing

Contributions are open for everyone. Feel free to just make a PR. However we do reject "vibecoding". The majority of the code must be made by yourself and any AI generated code must be vetted to ensure code quality.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
