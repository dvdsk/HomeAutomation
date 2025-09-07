#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr, StackResources};
use embassy_net_wiznet::{chip::W5500, Device, Runner, State};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Config;
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use heapless::Vec;
use static_cell::StaticCell;

use defmt::{trace, unwrap};
use {defmt_rtt as _, panic_probe as _};

mod network;
mod rng;

type EthernetSPI = ExclusiveDevice<Spi<'static, Async>, Output<'static>, Delay>;
#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W5500,
        EthernetSPI,
        ExtiInput<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(
    mut runner: embassy_net::Runner<'static, Device<'static>>,
) -> ! {
    runner.run().await
}

// 84 Mhz clock stm32f401
fn config() -> Config {
    use embassy_stm32::rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv,
        PllPreDiv, PllSource, Sysclk,
    };

    let mut config = Config::default();
    config.rcc.hse = Some(Hse {
        freq: Hertz(25_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV25,
        mul: PllMul::MUL336,
        divp: Some(PllPDiv::DIV4),
        divq: None,
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.sys = Sysclk::PLL1_P;
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(config());
    let mut dog = IndependentWatchdog::new(
        p.IWDG, 10_000_000, // microseconds
    );
    dog.unleash();
    let seed = rng::generate_seed_blocking(p.ADC1).await;
    defmt::info!("random seed: {}", seed);

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = Hertz(5_000_000); // Up to 50m works
    let (miso, mosi, clk) = (p.PA6, p.PA7, p.PA5);
    let spi =
        Spi::new(p.SPI1, clk, mosi, miso, p.DMA2_CH3, p.DMA2_CH0, spi_cfg);
    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi = unwrap!(ExclusiveDevice::new(spi, cs, Delay));

    let w5500_int = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up);
    let w5500_reset = Output::new(p.PB1, Level::High, Speed::Medium);

    let mac_addr = [0x02, 234, 3, 4, 82, 231];
    static STATE: StaticCell<State<3, 2>> = StaticCell::new();
    let state = STATE.init(State::<3, 2>::new());
    let (device, runner) = unwrap!(
        embassy_net_wiznet::new(mac_addr, state, spi, w5500_int, w5500_reset)
            .await
    );
    unwrap!(spawner.spawn(ethernet_task(runner)));

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let config =
        embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 7), 24),
            gateway: Some(Ipv4Address::new(192, 168, 1, 1)),
            dns_servers: Vec::new(),
        });
    let (stack, runner) = embassy_net::new(
        device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    // Launch network task
    unwrap!(spawner.spawn(net_task(runner)));

    let handle_network = network::handle(stack);
    spawner.must_spawn(handle_network);

    keep_dog_happy(dog).await;
}

async fn keep_dog_happy(mut dog: IndependentWatchdog<'_, IWDG>) {
    loop {
        Timer::after_secs(8).await;
        trace!("petting dog");
        dog.pet();
    }
}
