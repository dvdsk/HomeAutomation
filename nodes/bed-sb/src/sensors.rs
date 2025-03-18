use defmt::info;
use embassy_futures::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use fast::ButtonInputs;
use protocol::small_bedroom::bed::Device;

use crate::channel::Queues;
use sensors::{Error, SPS30_UART_BUF_SIZE};
pub mod fast;
pub mod slow;

use sensors::{
    Bme680Driver, Max44Driver, Nau7802Driver, Sht31Driver, Sps30Driver, MhzDriver
};

fn local_dev(dev: protocol::small_bedroom::bed::Device) -> protocol::Device {
    protocol::Device::SmallBedroom(protocol::small_bedroom::Device::Bed(dev))
}

pub async fn init_then_measure(
    publish: &Queues,
    orderers: &slow::DriverOrderers,
    i2c_1: Mutex<NoopRawMutex, I2c<'static, Async>>,
    i2c_3: Mutex<NoopRawMutex, I2c<'static, Async>>,
    usart_mhz: Uart<'static, Async>,
    usart_sps: Uart<'static, Async>,
    buttons: ButtonInputs,
) -> Result<(), Error> {
    info!("initializing sensors");
    let bme = Bme680Driver::new(&i2c_3, local_dev(Device::Bme680));
    let max44009 = Max44Driver::new(&i2c_1, local_dev(Device::Max44));
    let sht = Sht31Driver::new(&i2c_1, local_dev(Device::Sht31));
    let nau = Nau7802Driver::new(&i2c_3, local_dev(Device::Nau7802));

    let (tx, rx) = usart_mhz.split();
    let mut usart_buf = [0u8; 9 * 10]; // 9 byte messages
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let mhz = MhzDriver::new(tx, rx, local_dev(Device::Mhz14));

    let (tx, rx) = usart_sps.split();
    let mut usart_buf = [0u8; SPS30_UART_BUF_SIZE];
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let sps30 = Sps30Driver::init(tx, rx, local_dev(Device::Sps30));

    let sensors_fast = fast::read(max44009, nau, buttons, publish);
    let drivers = slow::Drivers {
        sht,
        bme,
        mhz,
        sps: sps30,
    };
    let sensors_slow = slow::read(drivers, orderers, publish);
    join::join(sensors_fast, sensors_slow).await;

    defmt::unreachable!();
}
