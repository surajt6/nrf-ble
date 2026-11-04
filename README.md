# nRF52840 BLE Template

A cargo-generate template for nRF52840 BLE applications using Embassy, the
Nordic SoftDevice Controller (`nrf-sdc`), and TrouBLE. Choose **scanner** or
**peripheral** during generation; only the selected application is included.
There is no default mode.

Scanner mode logs discovered addresses via RTT and highlights advertisements
matching `MY_DEVICE_ADDRESS`. Peripheral mode is based on `ble-button`, with
encrypted battery/status characteristics and a persistent bond.

## Prerequisites

- Rust with the `thumbv7em-none-eabi` target installed
- [cargo-generate](https://github.com/cargo-generate/cargo-generate) 0.25.0 or newer
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

The mode prompt requires `scanner` or `peripheral`. Peripheral mode also requires
these inputs, with no defaults:

| Input | Requirement |
| --- | --- |
| `advertised_name` | Nonempty, at most 22 UTF-8 bytes |
| `gatt_name` | Nonempty, at most 22 UTF-8 bytes |
| `device_address` | Static-random address in `AA:BB:CC:DD:EE:FF` notation |

The address must start with an octet in C0–FF, and its remaining 46 bits cannot
be all zero or all one. Choose a distinct address for each peripheral. Names
are preserved exactly; invalid inputs stop generation with an explanation.

For noninteractive generation, supply every required input explicitly:

```bash
cargo generate --git git@github.com:surajt6/nrf-ble.git \
    --name my-scanner --silent --define mode=scanner

cargo generate --git git@github.com:surajt6/nrf-ble.git \
    --name my-peripheral --silent --define mode=peripheral \
    --define advertised_name="My Peripheral" --define gatt_name="My GATT" \
    --define device_address="FF:E4:05:1A:8F:FF"
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
    cargo b

# Flash and display application logs
env PATH="/opt/homebrew/opt/llvm/bin:$PATH" \
    LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
    CLANG_PATH=/opt/homebrew/opt/llvm/bin/clang \
    cargo r
```

On other systems, point these environment variables at the installed LLVM/Clang
tools. Press Ctrl-C to end the log session.

## Dependency and runtime configuration

- Embassy executor 0.10 uses `platform-cortex-m` and task creation returns the
  fallible spawn token.
- Embassy nRF 0.11, `nrf-sdc` 0.4, and TrouBLE 0.8 use published crates. The
  mode-specific lockfile records resolved versions. Peripheral mode also pins
  `embassy-crypto` to the same Git revision used by `ble-button`.
- Keep `bt-hci` on 0.10 to match the controller and host dependencies.
- TrouBLE 0.8 initializes its security RNG from the controller. Its stack builder,
  central handle, and scanner APIs are configured in `src/ble_scanner.rs`.
- The scanner controller requires a **2784-byte** buffer; the peripheral
  controller requires **1904 bytes**.
  If controller features or dependencies change, check the required size reported
  by `nrf-sdc` and adjust `sdc::Mem` accordingly.
- The probe runner uses `--no-catch-reset`: Embassy can reset after updating UICR
  configuration on the first boot after an erase. Without this option, probe-rs
  can stop at the reset entry point (`0x100`) with `Firmware exited unexpectedly:
  Exception` and an unwind error. HardFault detection remains enabled.

The firmware links the controller and MPSL directly and does not require a
separately flashed S140 SoftDevice image. Both modes use 256 KiB RAM. Scanner
mode uses the full 1 MiB flash; peripheral mode reserves the final 8 KiB
(`0xFE000..0x100000`) for bond storage and uses MPSL timeslots for flash writes.

Peripheral mode restores one stored bond at boot and saves a new bond after
pairing. Its central must initiate encryption; notifications wait for encryption
to complete, including after reconnecting. The example sends a counter through
the battery-level characteristic every second and logs RSSI and connection
metrics. Its custom status characteristic supports encrypted reads and writes.
Disconnecting returns the device to advertising. A full chip erase clears the
stored bond; ordinary firmware updates must preserve the storage region.

The peripheral CSPRNG is seeded from the hardware RNG before BLE startup.

## Customization

In scanner mode, set `MY_DEVICE_ADDRESS` in `src/ble_scanner.rs` to the device to highlight. The
`BdAddr` byte array is least-significant-byte first, the reverse of the usual
printed address. All discovered addresses are still logged.

The scanner uses a fixed random local address for development. Give each device
an appropriate unique address when adapting the template for deployment.

In peripheral mode, edit the generated name strings and random address in
`src/ble_peripheral.rs` to change its identity later. Service UUIDs and behavior
match the source example.

## Generated project structure

```text
Cargo.toml           Dependencies and generated package metadata
Cargo.lock           Resolved dependency versions
.cargo/config.toml   Target, probe runner, and logging configuration
memory.x             nRF52840 flash and RAM layout
build.rs             Linker configuration
src/main.rs          Embassy, MPSL, and controller initialization
src/ble_scanner.rs   Scanner mode only: advertisement logging
src/ble_peripheral.rs Peripheral mode only: GATT, bonding, notifications
src/csprng.rs         Peripheral mode only: hardware-seeded RNG driver
```

## Validate local template changes

```bash
cargo generate --path . --name template-check
cd template-check
```

Generate each mode, run the build command above, then `cargo fmt --check`.
The template regression checks run with `python3 tests/check_generation.py`;
they require cargo-generate but do not build or flash firmware.

Template maintainers: the root firmware files are the scanner variant;
`.template/peripheral/` contains the peripheral variant. The pre-generation hook
selects the files and validates identity inputs without running shell commands.
Generated projects contain neither hooks nor the template test suite.

Hardware behavior requires a connected nRF52840, debug probe, and BLE central;
generation and build checks do not validate radio behavior.
