//! Board support for the Seeed XIAO nRF54L15 (Sense).
//!
//! Pin assignments and crystal load capacitance follow Seeed's Zephyr board definition
//! (`boards/seeed/xiao_nrf54l15`).

use embassy_nrf::Peri;
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use embassy_nrf::pac;
use embassy_nrf::peripherals::P2_00;
use embassy_time::{Duration, Timer};

/// Internal load capacitance for the 32.768 kHz and 32 MHz crystals, in femtofarads.
const LOAD_CAPACITANCE_FF: u32 = 16_000;

/// Decodes a 9-bit two's-complement FICR trim slope.
fn trim_slope(field: u16) -> i32 {
    ((field as i32) ^ 0x100) - 0x100
}

fn round_div(value: u32, divisor: u32) -> u32 {
    value / divisor + u32::from(value % divisor >= divisor / 2)
}

/// Port of nrfx `FICR_LFXO_CAP_CALCULATE` for the nRF54L series.
fn lfxo_intcap(cap_ff: u32) -> u8 {
    let trim = pac::FICR.xosc32ktrim().read();
    let slope = (trim_slope(trim.slope()) + 392) as u32;
    let offset = u32::from(trim.offset());
    round_div((2 * cap_ff - 12_000) * slope + (offset << 3) * 1000, 512_000) as u8
}

/// Port of nrfx `FICR_HFXO_CAP_CALCULATE` for the nRF54L series.
fn hfxo_intcap(cap_ff: u32) -> u8 {
    let trim = pac::FICR.xosc32mtrim().read();
    let slope = (trim_slope(trim.slope()) + 791) as u32;
    let offset = u32::from(trim.offset());
    round_div(((cap_ff - 5_500) * slope + (offset << 2) * 1000) >> 8, 1000) as u8
}

/// Configures the crystal oscillators' internal load capacitors.
///
/// Call before `embassy_nrf::init`, which starts the crystals.
pub fn configure_crystals() {
    let lfxo = lfxo_intcap(LOAD_CAPACITANCE_FF);
    let hfxo = hfxo_intcap(LOAD_CAPACITANCE_FF);
    pac::OSCILLATORS.xosc32ki().intcap().write(|w| w.set_val(lfxo));
    pac::OSCILLATORS
        .xosc32m()
        .config()
        .intcap()
        .write(|w| w.set_val(hfxo));
}

/// Board outputs that must stay driven for the lifetime of the application.
pub struct Board {
    _rf_switch_power: Output<'static>,
    _rf_switch_select: Output<'static>,
    _sensor_power: Output<'static>,
}

/// Powers the RF switch, selects the onboard ceramic antenna, and turns off the
/// IMU and microphone supply.
pub fn init(
    rf_switch_power: Peri<'static, impl Pin>,
    rf_switch_select: Peri<'static, impl Pin>,
    sensor_power: Peri<'static, impl Pin>,
) -> Board {
    Board {
        _rf_switch_power: Output::new(rf_switch_power, Level::High, OutputDrive::Standard),
        // Low selects the ceramic antenna; High selects the U.FL connector.
        _rf_switch_select: Output::new(rf_switch_select, Level::Low, OutputDrive::Standard),
        _sensor_power: Output::new(sensor_power, Level::Low, OutputDrive::Standard),
    }
}

/// Blinks the active-low user LED as an alive indicator.
#[embassy_executor::task]
pub async fn blink_led(led: Peri<'static, P2_00>) -> ! {
    let mut led = Output::new(led, Level::High, OutputDrive::Standard);
    loop {
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;
        led.set_high();
        Timer::after(Duration::from_millis(950)).await;
    }
}
