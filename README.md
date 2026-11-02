# nRF52840 BLE Template

A cargo-generate template for an nRF52840 BLE scanner using Embassy, the Nordic
SoftDevice Controller (`nrf-sdc`), and TrouBLE. It logs discovered device addresses
via RTT and highlights advertisements matching `MY_DEVICE_ADDRESS` in
`src/ble_scanner.rs`.

## Prerequisites

- Rust with the `thumbv7em-none-eabi` target installed
- [cargo-generate](https://github.com/cargo-generate/cargo-generate)
- [probe-rs](https://probe.rs/) and a connected SWD debug probe
- LLVM/Clang for generating the Nordic controller bindings

```bash
rustup target add thumbv7em-none-eabi
cargo install cargo-generate

# On Apple Silicon macOS
brew install llvm
```

## Generate a project

```bash
cargo generate --git git@github.com:surajt6/nrf-ble.git --name my-ble-scanner
cd my-ble-scanner
```

Generation fills in the package name, binary name, lockfile package name, and
author metadata. Build the generated project; the template checkout contains
placeholders.

## Build and run

On Apple Silicon macOS, use Homebrew's LLVM tools:

```bash
# Build without connecting to hardware
env PATH="/opt/homebrew/opt/llvm/bin:$PATH" \
    LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
    CLANG_PATH=/opt/homebrew/opt/llvm/bin/clang \
    cargo build --locked

# Flash and display BLE scan logs
env PATH="/opt/homebrew/opt/llvm/bin:$PATH" \
    LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
    CLANG_PATH=/opt/homebrew/opt/llvm/bin/clang \
    cargo r --locked
```

Add `--release` for an optimized build. On other systems, point these environment
variables at the installed LLVM/Clang tools. Press Ctrl-C to end the log session.

## Dependency and runtime configuration

- Embassy executor 0.10 uses `platform-cortex-m` and task creation returns the
  fallible spawn token.
- Embassy nRF 0.11, `nrf-sdc` 0.4, and TrouBLE 0.8 use published crates. The
  lockfile records the resolved versions; no Git patches are required.
- Keep `bt-hci` on 0.10 to match the controller and host dependencies.
- TrouBLE 0.8 initializes its security RNG from the controller. Its stack builder,
  central handle, and scanner APIs are configured in `src/ble_scanner.rs`.
- The controller configuration in `build_sdc` requires a **2784-byte** buffer.
  If controller features or dependencies change, check the required size reported
  by `nrf-sdc` and adjust `sdc::Mem` accordingly.
- The probe runner uses `--no-catch-reset`: Embassy can reset after updating UICR
  configuration on the first boot after an erase. Without this option, probe-rs
  can stop at the reset entry point (`0x100`) with `Firmware exited unexpectedly:
  Exception` and an unwind error. HardFault detection remains enabled.

The firmware links the controller and MPSL directly. It uses the nRF52840's full
1 MB flash / 256 KB RAM layout starting at flash address zero, and does not
require a separately flashed S140 SoftDevice image.

## Customization

Set `MY_DEVICE_ADDRESS` in `src/ble_scanner.rs` to the device to highlight. The
`BdAddr` byte array is least-significant-byte first, the reverse of the usual
printed address. All discovered addresses are still logged.

The scanner uses a fixed random local address for development. Give each device
an appropriate unique address when adapting the template for deployment.

## Project structure

```text
Cargo.toml           Dependencies and generated package metadata
Cargo.lock           Resolved dependency versions
.cargo/config.toml   Target, probe runner, and logging configuration
memory.x             nRF52840 flash and RAM layout
build.rs             Linker configuration
src/main.rs          Embassy, MPSL, and controller initialization
src/ble_scanner.rs   BLE host, scanning, and advertisement logging
```

## Validate local template changes

```bash
cargo generate --path . --name template-check
cd template-check
```

Run the build commands above, then `cargo fmt --check`. Hardware scanning requires
a connected nRF52840 and debug probe.
