use bt_hci::cmd::le::LeSetExtScanParams;
use bt_hci::controller::ControllerCmdSync;
use core::cell::RefCell;
use defmt::*;
use embassy_futures::join::join;
use embassy_time::{Duration, Timer};
use heapless::Deque;
use trouble_host::prelude::*;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

const MY_DEVICE_ADDRESS: BdAddr = BdAddr([0xb5, 0x91, 0xe8, 0x02, 0x50, 0x3c]);

pub async fn run<C>(controller: C)
where
    C: Controller + ControllerCmdSync<LeSetExtScanParams>,
{
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);

    info!("Our address = {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack: Stack<'_, C, _> = trouble_host::new(controller, &mut resources)
        .set_random_address(address)
        .set_io_capabilities(IoCapabilities::NoInputNoOutput)
        .build();

    let mut central = stack.central();
    let mut runner = stack.runner();

    let printer = Printer {
        seen: RefCell::new(Deque::new()),
    };
    let mut scanner = Scanner::new(&mut central);
    let _ = join(runner.run_with_handler(&printer), async {
        let config = ScanConfig::default();
        info!("Starting Scan!");
        let mut _session = scanner.scan_ext(&config).await.unwrap();
        info!("Returned from scanner call!");
        // Scan forever
        loop {
            Timer::after(Duration::from_millis(100)).await;
        }
    })
    .await;
}

struct Printer {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl EventHandler for Printer {
    fn on_ext_adv_reports(&self, mut it: LeExtAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();
        while let Some(Ok(report)) = it.next() {
            if report.addr == MY_DEVICE_ADDRESS {
                info!("Advertising event from my device: {:?}", report.event_kind);
            }
            if seen.iter().find(|b| b.raw() == report.addr.raw()).is_none() {
                let bytes: &[u8] = report.addr.raw();
                info!(
                    "Discovered other device with address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0]
                );
                if seen.is_full() {
                    seen.pop_front();
                }
                seen.push_back(report.addr).unwrap();
            }
        }
    }
}
