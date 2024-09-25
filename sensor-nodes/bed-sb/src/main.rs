#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{self, Either3};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{self, DataBits, Parity, StopBits, Uart};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::{usb, Config};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use futures::pin_mut;
use sensors::fast::ButtonInputs;
use sensors::slow;

use defmt::{error, trace, unwrap};
use {defmt_rtt as _, panic_probe as _};

mod channel;
mod comms;
mod error_cache;
mod sensors;
mod usb_wrapper;
use crate::channel::Queues;

embassy_stm32::bind_interrupts!(struct Irqs {
    I2C1_EV => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C3_EV => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C3>;
    I2C3_ER => embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C3>;
    USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;
    USART2 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
    OTG_FS => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB_OTG_FS>;
});

// todo use nusb to write driver

// 84 Mhz clock stm32f401
fn config() -> Config {
    use embassy_stm32::rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource,
        Sysclk,
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
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());

    let mut usart_config = usart::Config::default();
    usart_config.baudrate = 9600;
    usart_config.data_bits = DataBits::DataBits8;
    usart_config.stop_bits = StopBits::STOP1;
    let usart_mhz = unwrap!(Uart::new(
        p.USART1,
        p.PB7,
        p.PB6,
        Irqs,
        p.DMA2_CH7,
        p.DMA2_CH2,
        usart_config,
    ));

    let mut usart_config = usart::Config::default();
    usart_config.baudrate = 115_200; // sps30 only works at this baud
    usart_config.data_bits = DataBits::DataBits8;
    usart_config.stop_bits = StopBits::STOP1;
    usart_config.parity = Parity::ParityNone;
    let usart_sps30 = unwrap!(Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH5,
        usart_config,
    ));

    let i2c_1 = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH7,
        p.DMA1_CH0,
        Hertz(150_000),
        i2c::Config::default(),
    );
    let i2c_1: Mutex<NoopRawMutex, _> = Mutex::new(i2c_1);

    let i2c_3 = I2c::new(
        p.I2C3,
        p.PA8,
        p.PB4,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH1,
        // Extra slow, helps with longer cable runs
        Hertz(150_000),
        i2c::Config::default(),
    );
    let i2c_3: Mutex<NoopRawMutex, _> = Mutex::new(i2c_3);

    let buttons = ButtonInputs {
        left: ExtiInput::new(p.PC14, p.EXTI14, Pull::Down),
        left_middle: ExtiInput::new(p.PA9, p.EXTI9, Pull::Down),
        right_middle: ExtiInput::new(p.PC13, p.EXTI13, Pull::Down),
        right: ExtiInput::new(p.PA10, p.EXTI10, Pull::Down),
    };

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

    let stack_a = usb_wrapper::StackA::new();
    let mut stack_b = usb_wrapper::StackB::new(&stack_a);
    let (mut usb_bus, usb_handle) = usb_wrapper::new(&stack_a, &mut stack_b, usb_driver);

    let driver_orderers = slow::DriverOrderers::new();
    let publish = Queues::new();
    let handle_network = comms::handle(usb_handle, &publish, &driver_orderers);
    pin_mut!(handle_network);

    let init_then_measure = sensors::init_then_measure(
        &publish,
        &driver_orderers,
        i2c_1,
        i2c_3,
        usart_mhz,
        usart_sps30,
        buttons,
    );

    let res = select::select3(&mut handle_network, init_then_measure, usb_bus.run()).await;
    let unrecoverable_err = match res {
        Either3::First(()) | Either3::Third(()) | Either3::Second(Ok(())) => defmt::unreachable!(),
        Either3::Second(Err(err)) => err,
    };

    // at this point no other errors have occurred
    error!("unrecoverable error, resetting: {}", unrecoverable_err);
    publish.queue_error(unrecoverable_err);
    handle_network.await; // if this takes too long the dog will get us
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

async fn keep_dog_happy(mut dog: IndependentWatchdog<'_, IWDG>) {
    loop {
        dog.unleash();
        Timer::after_secs(8).await;
        trace!("petting dog");
        dog.pet();
    }
}
