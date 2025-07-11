# Contributing to Kulfi & Malai

It is recommended to first open a [discussion](https://github.com/fastn-stack/kulfi/discussions) to discuss your ideas, before opening a pull request. This helps us to understand your changes better, and also helps you to get feedback on your changes before you spend time on implementing them.

## Building from source

### Unix

You need to have `rust` and `cargo` installed. Install them using [rustup](https://rustup.rs/).

```bash
cargo build --bin malai
# build kulfi browser (tauri app)
cargo build --bin kulfi
```

### [Nix](https://nixos.org/) users

```bash
nix develop
# build malai cli
cargo build --bin malai
# build kulfi browser (tauri app)
cargo build --bin kulfi
```

By default, `cargo build` will build debug binaries. This, for the tauri app, lets you open the dev tools. Browser dev tools are not available on a release build of `kulfi`.

## Running

You can run the `malai` cli by using the following command:

```bash
./target/debug/malai --help
```

To run the kulfi UI app:

```bash
./target/debug/kulfi
```
