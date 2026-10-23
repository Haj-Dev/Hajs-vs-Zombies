# Hajs vs Zombies

Hajs vs Zombies is a 3ds tower defense game, built with Rust thanks to the amazing [Rust3DS](https://github.com/rust3ds/) project.

# Environment
Required tools:
- A recent rust version
- The nightly-x86_64-unknown-linux-gnu toolchain rust-src component
- devkitPro 3DS tools
- cargo-3ds
- Some version of llvm providing `bindgen`

To get started, install the [rust programming language](https://rust-lang.org/) on your machine. Then, follow the [devkitPro installation guide](https://devkitpro.org/wiki/devkitPro_pacman) and install the `3ds-dev` group.
You'll also have to ensure that `$DEVKITPRO/tools/bin` and `$DEVKITARM/bin` are in your `PATH`.

You can then proceed to the installation of cargo-3ds. The latest git version is required.
```sh
cargo install --locked --git https://github.com/rust3ds/cargo-3ds
```

You also need to install the nightly-x86_64-unknown-linux-gnu toolchain rust-src component:
```sh
rustup toolchain install nightly-x86_64-unknown-linux-gnu
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
```

Finally, you can install some version of llvm to get access to `bindgen`.

# Building the project
As we are developping a game made for 3ds, you'll need to use `cargo-3ds` commands. Cargo 3DS is simple to use; you can simply run usual cargo commands, with the `cargo-3ds` binary replacing `cargo`. For example, to build the project in debug mode, you can use:
```sh
cargo 3ds build
```
To build the project in release mode, you can use:
```sh
cargo 3ds build --release
```

These commands will produce a `3dsx` file in the `target/armv6k-nintendo-3ds/debug` or `target/armv6k-nintendo-3ds/release` directory, depending on the mode.
You can then run the `3dsx` file on a 3DS emulator or on a real device.

For debugging instructions, follow the [Rust3DS documentation](https://github.com/rust3ds/ctru-rs/wiki/Guides#debugging-using-gdb).

# Running
There are two ways to run the project: using the `3dsx` file directly after building, either on a 3DS emulator or on a real device, or using `cargo 3ds run`.

Cargo 3ds allows you to run the project directly on your nintendo 3DS device, without needing to manually transfer the `3dsx` file to it.
To do that, open the homebrew menu on your 3DS and press Y.
This will start the `3dslink` pairing.

You can then use
```sh
cargo 3ds run
```
to build and run the project directly on your 3DS.
If your Nintendo 3DS is not automatically found, you can specify its IP address using the `--address` flag:
```sh
cargo 3ds run --address <ip_address>
```
For more information, see the [cargo-3ds documentation](https://github.com/rust3ds/cargo-3ds#usage).
