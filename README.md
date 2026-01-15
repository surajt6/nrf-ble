# nRF52840 Embassy Template

A cargo-generate template for embedded Rust projects targeting the Nordic nRF52840 microcontroller using the Embassy async framework.

## Prerequisites

- [Rust](https://rustup.rs/) with `thumbv7em-none-eabi` target installed
- [probe-rs](https://probe.rs/) for flashing and debugging
- [cargo-generate](https://github.com/cargo-generate/cargo-generate) for creating projects from this template

### Installation

```bash
# Install the ARM Cortex-M4F target
rustup target add thumbv7em-none-eabi

# Install probe-rs
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-installer.sh | sh

# Install cargo-generate
cargo install cargo-generate
```

## Usage

Generate a new project from this template:

```bash
cargo generate --git https://github.com/yourusername/nrf52-embassy-template
```

You will be prompted for:
- **Project Name**: Your project name (e.g., `my-nrf-project`)
- **Authors**: Your name and email
- **Enable Nordic SoftDevice**: Whether to configure memory layout for Bluetooth SoftDevice S140 7.3.0

## What's Included

The generated project contains:

- **LED Blink Example**: Simple async application that blinks an LED on pin P0_13
- **Embassy Framework**: Modern async/await runtime for embedded Rust
- **defmt Logging**: Efficient logging with RTT transport
- **Memory Configuration**: Automatically configured based on SoftDevice selection
- **probe-rs Integration**: Ready to flash and debug with `cargo run`

## Hardware

This template targets the **Nordic nRF52840** microcontroller:
- ARM Cortex-M4F core
- 1 MB Flash / 256 KB RAM (without SoftDevice)
- 868 KB Flash / 128 KB RAM (with SoftDevice S140 7.3.0)

Tested with development boards like:
- nRF52840-DK
- nRF52840 Dongle
- Other nRF52840-based boards

## Building and Running

```bash
# Build the project
cargo build --release

# Flash to device and view logs
cargo run --release
```

The application will blink the LED on pin P0_13 and output "Blink!" messages via RTT.

## Project Structure

```
.
├── Cargo.toml              # Project dependencies and metadata
├── .cargo/
│   └── config.toml         # Build configuration and probe-rs runner
├── memory.x                # Memory layout (varies based on SoftDevice)
├── build.rs                # Build script for linker configuration
└── src/
    └── main.rs             # Main application code
```

## Customization

### Changing the LED Pin

Edit `src/main.rs:13` to use a different GPIO pin:

```rust
let mut led = Output::new(p.P0_06, Level::Low, OutputDrive::Standard);
```

### Adding Dependencies

Add Embassy HAL peripherals or other crates to `Cargo.toml`:

```toml
embassy-nrf = { version = "0.9.0", features = ["defmt", "nrf52840", "time-driver-rtc1", "gpiote"] }
```

### Using SoftDevice (Bluetooth)

If you selected SoftDevice support during generation, the memory layout is pre-configured for SoftDevice S140 7.3.0. You'll need to:

1. Flash the SoftDevice to your device first
2. Add the `nrf-softdevice` crate for Bluetooth functionality
3. Update your code to initialize the SoftDevice

## License

This template is provided as-is for use in your projects.

## Resources

- [Embassy Documentation](https://embassy.dev/)
- [nRF52840 Product Specification](https://www.nordicsemi.com/Products/nRF52840)
- [probe-rs Documentation](https://probe.rs/)
- [defmt Book](https://defmt.ferrous-systems.com/)
