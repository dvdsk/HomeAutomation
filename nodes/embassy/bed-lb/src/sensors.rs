use defmt::info;
use embassy_executor::task;
use embassy_futures::join;
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use fast::ButtonInputs;
use protocol::large_bedroom::bed::Device;

use crate::rgb_led;
use sensors::{I2cWrapper, Nau7802DriverBlocking, SPS30_UART_BUF_SIZE};
pub mod fast;
pub mod slow;

use sensors::{
    Bme680Driver, Max44Driver, MhzDriver, Nau7802Driver, Sht31Driver,
    Sps30Driver,
};

fn local_dev(dev: protocol::large_bedroom::bed::Device) -> protocol::Device {
    protocol::Device::LargeBedroom(protocol::large_bedroom::Device::Bed(dev))
}

#[task]
pub async fn init_then_measure(
    orderers: &'static slow::DriverOrderers,
    rgb_led: rgb_led::LedHandle,
    i2c_1: Mutex<NoopRawMutex, I2c<'static, Async, Master>>,
    i2c_2: Mutex<NoopRawMutex, I2c<'static, Blocking, Master>>,
    i2c_3: Mutex<NoopRawMutex, I2c<'static, Async, Master>>,
    usart_mhz: Uart<'static, Async>,
    usart_sps: Uart<'static, Async>,
    buttons: ButtonInputs,
) -> ! {
    info!("initializing sensors");
    let bme = Bme680Driver::new(&i2c_3, local_dev(Device::Bme680));
    let max44009 = Max44Driver::new(&i2c_1, local_dev(Device::Max44));
    let sht = Sht31Driver::new(&i2c_1, local_dev(Device::Sht31));
    let i2c_2 = I2cWrapper(i2c_2);
    let nau_right =
        Nau7802DriverBlocking::new(&i2c_2, local_dev(Device::Nau7802Right));
    let nau_left = Nau7802Driver::new(&i2c_3, local_dev(Device::Nau7802Left));

    let (tx, rx) = usart_mhz.split();
    let mut usart_buf = [0u8; 9 * 10]; // 9 byte messages
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let mhz = MhzDriver::new(tx, rx, local_dev(Device::Mhz14));

    let (tx, rx) = usart_sps.split();
    let mut usart_buf = [0u8; SPS30_UART_BUF_SIZE];
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let sps30 = Sps30Driver::init(tx, rx, local_dev(Device::Sps30));

    let sensors_fast =
        fast::read(max44009, nau_right, nau_left, buttons, rgb_led);
    let drivers = slow::Drivers {
        sht,
        bme,
        mhz,
        sps: sps30,
    };
    let sensors_slow = slow::read(drivers, orderers);
    join::join(sensors_fast, sensors_slow).await;

    defmt::unreachable!();
}
