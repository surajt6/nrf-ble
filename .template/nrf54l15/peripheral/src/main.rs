#![no_std]
#![no_main]

use core::ops::Range;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::cracen::Cracen;
use embassy_nrf::mode::Blocking;
use embassy_nrf::{bind_interrupts, config};
use embassy_time::{Duration, Timer};
use nrf_sdc::mpsl::Flash;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
mod ble_peripheral;
mod csprng;
{%- if xiao_rf_switch %}
mod xiao;
{%- endif %}

unsafe extern "C" {
    static __storage_start: u8;
    static __storage_end: u8;
}

fn storage_range() -> Range<u32> {
    unsafe {
        let start = &__storage_start as *const u8 as u32;
        let end = &__storage_end as *const u8 as u32;
        start..end
    }
}

bind_interrupts!(struct Irqs {
    SWI00 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO_0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER10 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    GRTC_3 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut Cracen<'_, Blocking>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .peripheral_count(1)?
        .adv_count(1)?
        .build(p, rng, mpsl, mem)
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
{%- if xiao_rf_switch %}
    xiao::configure_crystals();
{%- endif %}
    let mut config = config::Config::default();
    // MPSL requires the 128 MHz CPU clock.
    config.clock_speed = config::ClockSpeed::CK128;
    config.hfclk_source = config::HfclkSource::ExternalXtal;
    config.lfclk_source = config::LfclkSource::ExternalXtal;
    let p = embassy_nrf::init(config);
{%- if xiao_rf_switch %}
    let _board = xiao::init(p.P2_03, p.P2_05, p.P0_01);
    spawner.spawn(xiao::blink_led(p.P2_00).unwrap());
{%- endif %}

    let mpsl_p = mpsl::Peripherals::new(
        p.GRTC_CH7,
        p.GRTC_CH8,
        p.GRTC_CH9,
        p.GRTC_CH10,
        p.GRTC_CH11,
        p.TIMER10,
        p.TIMER20,
        p.TEMP,
        p.PPI10_CH0,
        p.PPI20_CH1,
        p.PPIB11_CH0,
        p.PPIB21_CH0,
    );
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_XTAL as u8,
        rc_ctiv: 0,
        rc_temp_ctiv: 0,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    static SESSION_MEM: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
    let mpsl = MPSL.init(unwrap!(mpsl::MultiprotocolServiceLayer::with_timeslots(
        mpsl_p,
        Irqs,
        lfclk_cfg,
        SESSION_MEM.init(mpsl::SessionMem::new())
    )));
    spawner.spawn(mpsl_task(&*mpsl).unwrap());

    let sdc_p = sdc::Peripherals::new(
        p.PPI00_CH1,
        p.PPI00_CH3,
        p.PPI10_CH1,
        p.PPI10_CH2,
        p.PPI10_CH3,
        p.PPI10_CH4,
        p.PPI10_CH5,
        p.PPI10_CH6,
        p.PPI10_CH7,
        p.PPI10_CH8,
        p.PPI10_CH9,
        p.PPI10_CH10,
        p.PPI10_CH11,
        p.PPIB00_CH1,
        p.PPIB00_CH2,
        p.PPIB00_CH3,
        p.PPIB10_CH1,
        p.PPIB10_CH2,
        p.PPIB10_CH3,
    );

    let mut rng = Cracen::new_blocking(p.CRACEN);
    let mut seed = [0u8; 32];
    rng.blocking_fill_bytes(&mut seed);
    csprng::seed(seed);

    // Oversized until measured on hardware: nrf-sdc logs the exact requirement
    // ("Memory buffer too big. N bytes needed") at startup.
    let mut sdc_mem = sdc::Mem::<8192>::new();
    let sdc = unwrap!(build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem));
    let mut flash = Flash::take(mpsl, p.RRAMC);

    Timer::after(Duration::from_millis(200)).await;

    ble_peripheral::run(sdc, &mut flash, storage_range()).await;
}
