# Hardware verification checklist

Generation and build checks (`python3 tests/check_generation.py`, `cargo b`) do not
exercise the radio, clocks, or storage. Run these checks on real boards after
changing chip support, dependencies, or the critical-section setup.

## Setup

```bash
cargo install probe-rs-tools
probe-rs list                      # the board's probe must be listed
probe-rs chip list | grep -i 54l15 # confirm the chip name used in .cargo/config.toml
```

Generate each variant to test, for example:

```bash
cargo generate --path . --name xiao-scanner --silent \
    --define chip=nrf54l15 --define xiao_rf_switch=true --define mode=scanner
cargo generate --path . --name xiao-peripheral --silent \
    --define chip=nrf54l15 --define xiao_rf_switch=true --define mode=peripheral \
    --define advertised_name="XIAO Test" --define gatt_name="XIAO GATT" \
    --define device_address="FF:E4:05:1A:8F:FF"
```

Then run `cargo r` in the project, using the LLVM environment from the README.

## nRF54L15 (Seeed XIAO nRF54L15 Sense)

| # | Check | How | Pass criteria | If it fails |
|---|-------|-----|---------------|-------------|
| 1 | Flashing | `cargo r` | Firmware starts and RTT logs appear | Run `probe-rs erase --chip nRF54L15 --allow-erase-all` once (the factory firmware may write-protect the NVM) |
| 2 | Clocks start | Startup logs | No MPSL assert or panic; `Our address = …` is logged | Check the 128 MHz setting and crystal sources in `main.rs`, and the INTCAP values in `xiao.rs` |
| 3 | Controller memory | Startup logs | `nrf-sdc` warns `Memory buffer too big. N bytes needed` | Set `sdc::Mem::<N>` in `src/main.rs` to N for this mode, then update `.template/nrf54l15[/peripheral]/src/main.rs` |
| 4 | Alive LED | Watch the board | User LED blinks once per second | Check P2.00 and the `blink_led` task |
| 5 | Scanner | Scanner mode | Nearby devices are logged | Check the RF switch (P2.03 high) |
| 6 | Antenna select | Compare RSSI of a nearby device with P2.05 low, then high | Low gives the better RSSI with the ceramic antenna | If high is better, the polarity is inverted: flip the level in `xiao.rs` and the README |
| 7 | Advertising / connect | Peripheral mode with nRF Connect (phone) | Device is visible with the configured name; connects | Check the RF switch and the SDC memory size |
| 8 | Pair / encrypt | Pair from nRF Connect | Battery notifications start after encryption | Check the CSPRNG seed (CRACEN) |
| 9 | Bond persists | Reset the board, then reconnect | Bond restored from RRAM; encryption resumes without re-pairing | Check `memory.x` STORAGE (`0x17B000`, 8K) and `nrf_mpsl::Flash` (RRAMC) |
| 10 | GRTC CC11 overlap | Keep a connection for at least 10 minutes and watch the logged timing | No disconnects, asserts, or stalled timers | The Embassy time driver and MPSL both use GRTC CC11; move to an embassy-nrf version with `time-driver-grtc-ccN` |
| 11 | Sensor power off | Optional: measure current | IMU/mic supply (P0.01) is off | Check `xiao::init` |

Generic nRF54L15 boards (`xiao_rf_switch=false`) need a 32.768 kHz crystal; run
checks 1–3, 5 and 7–10.

## nRF52840 regression

Both chips now use `nrf-mpsl`'s `critical-section-impl` instead of cortex-m's
`critical-section-single-core`.

| # | Check | Pass criteria |
|---|-------|---------------|
| 1 | Scanner mode | Devices are logged; no MPSL asserts with `DEFMT_LOG=trace` |
| 2 | Peripheral mode | Connect, pair, notifications work |
| 3 | Bond persists | Reset, then reconnect without re-pairing |
| 4 | Stability | Connection held for at least 10 minutes |

## Record results

Note the board, date, commit, and any adjusted values (SDC memory sizes, antenna
polarity) in the commit that applies them.
