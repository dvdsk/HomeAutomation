#![no_std]
#![no_main]

use embassy_futures::join::join3;
use embassy_net::StackResources;
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use esp_println as _;

use embassy_executor::Spawner;

use esp_backtrace as _;
use esp_wifi::EspWifiController;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use protocol::large_bedroom::airbox::Error;

extern crate alloc;

mod fans;
mod network;
mod sensor;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> =
            static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

type Queue =
    Channel<NoopRawMutex, Result<protocol::Reading, protocol::Error>, 20>;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    defmt::warn!("started");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let fans = fans::Fans::new(peripherals.LEDC, peripherals.GPIO3);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK).unwrap()
    );

    let (controller, interfaces) =
        esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let wifi_interface = interfaces.sta;

    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(network::connection(controller)).ok();
    spawner.spawn(network::net_task(runner)).ok();

    let queue = Channel::new();
    join3(
        network::handle(&stack, &queue, &fans),
        sensor::button(peripherals.GPIO10, &queue, &fans),
        sensor::measure(peripherals.I2C0, &queue),
    )
    .await;
}

fn wrap_error(error: Error) -> protocol::Error {
    protocol::Error::LargeBedroom(protocol::large_bedroom::Error::Airbox(error))
}
