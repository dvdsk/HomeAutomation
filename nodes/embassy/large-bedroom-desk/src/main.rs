#![no_std]
#![no_main]

use core::time::Duration;

use embassy_executor::Spawner;
use embassy_futures::join;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::time::Hertz;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::{Config, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use futures::pin_mut;

use defmt::trace;
use {defmt_rtt as _, panic_probe as _};

mod comms;

embassy_stm32::bind_interrupts!(struct Irqs {
    OTG_FS => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB_OTG_FS>;
});

static PUBLISH: Channel<ThreadModeRawMutex, protocol::Reading, 20> =
    Channel::new();

// 84 Mhz clock stm32f401
// 48 Mhz clock for usb
fn config() -> Config {
    use embassy_stm32::rcc::mux;
    use embassy_stm32::rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv,
        PllPreDiv, PllQDiv, PllSource, Sysclk,
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
        divp: Some(PllPDiv::DIV4), // 25mhz / 25 * 336 / 4 = 84Mhz.
        divq: Some(PllQDiv::DIV7), // 25mhz / 25 * 336 / 7 = 48Mhz.
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    config
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    let dog = IndependentWatchdog::new(
        p.IWDG,
        Duration::from_secs(5).as_micros() as u32,
    );

    use protocol::large_bedroom::desk_right;
    use protocol::large_bedroom::desk_right::Button;
    type IntoButton = fn(protocol::button::Press) -> Button;
    let buttons = [
        (
            ExtiInput::new(p.PC15, p.EXTI15, Pull::Down),
            Button::LeftLeft as IntoButton,
        ),
        (
            ExtiInput::new(p.PB0, p.EXTI0, Pull::Down),
            Button::LeftMiddle as IntoButton,
        ),
        (
            ExtiInput::new(p.PC14, p.EXTI14, Pull::Down),
            Button::LeftRight as IntoButton,
        ),
        (
            ExtiInput::new(p.PB12, p.EXTI12, Pull::Down),
            Button::RightLeftmost as IntoButton,
        ),
        (
            ExtiInput::new(p.PB13, p.EXTI13, Pull::Down),
            Button::RightLeft as IntoButton,
        ),
        (
            ExtiInput::new(p.PB8, p.EXTI8, Pull::Down),
            Button::RightRight as IntoButton,
        ),
        (
            ExtiInput::new(p.PB6, p.EXTI6, Pull::Down),
            Button::RightRightmost as IntoButton,
        ),
    ];

    let pirs = [(
        ExtiInput::new(p.PA5, p.EXTI5, Pull::Down),
        desk_right::Reading::Pir,
    )];

    let mut usb_driver_config = embassy_stm32::usb::Config::default();
    // Do not enable vbus_detection. This is a safe default that works in all
    // boards. However, if your USB device is self-powered (can stay powered on
    // if USB is unplugged), you need to enable vbus_detection to comply with
    // the USB spec. If you enable it, the board
    usb_driver_config.vbus_detection = false;

    let mut ep_out_buffer = [0u8; 256];
    let usb_driver = usb::Driver::new_fs(
        p.USB_OTG_FS,
        Irqs,
        p.PA12,
        p.PA11,
        &mut ep_out_buffer,
        usb_driver_config,
    );

    let affector_list = comms::affector_list();
    let stack_a = usb_bridge_client::StackA::new();
    let mut stack_b = usb_bridge_client::StackB::new(&stack_a, &affector_list);
    let (mut usb_bus, usb_handle) = usb_bridge_client::new(
        &stack_a,
        &mut stack_b,
        usb_bridge_client::config!("large-bedroom-righ", "87244"),
        usb_driver,
    );

    let handle_network = comms::handle(usb_handle);
    pin_mut!(handle_network);

    let watch_buttons = sensors::button::watch_array(buttons, |button| {
        let _err = PUBLISH.try_send(button_into_reading(button));
    });
    let watch_pir = sensors::pir::watch_array(pirs, |pir| {
        let _err = PUBLISH.try_send(into_reading(pir));
    });

    join::join5(
        &mut handle_network,
        watch_buttons,
        watch_pir,
        usb_bus.run(),
        keep_dog_happy(dog),
    )
    .await;
}

pub fn usb_config() -> embassy_usb::Config<'static> {
    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("Vid");
    config.product = Some("Sensor node");
    config.serial_number = Some("2478437");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}

async fn keep_dog_happy(mut dog: IndependentWatchdog<'_, IWDG>) -> ! {
    loop {
        Timer::after_secs(1).await;
        trace!("petting dog");
        dog.pet();
    }
}

fn button_into_reading(
    button: protocol::large_bedroom::desk_right::Button,
) -> protocol::Reading {
    protocol::Reading::LargeBedroom(
        protocol::large_bedroom::Reading::DeskRight(
            protocol::large_bedroom::desk_right::Reading::Button(button),
        ),
    )
}

fn into_reading(
    pir: protocol::large_bedroom::desk_right::Reading,
) -> protocol::Reading {
    protocol::Reading::LargeBedroom(
        protocol::large_bedroom::Reading::DeskRight(pir),
    )
}
