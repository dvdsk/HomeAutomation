use defmt::info;
use embassy_embedded_hal::shared_bus::I2cDeviceError;
use embassy_futures::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{with_timeout, Delay, Duration};
use mhzx::MHZ;
use protocol::large_bedroom::bed::Device;
use sps30_async::Sps30;

use crate::channel::Queues;
use crate::error_cache::{Error, SensorError};
use crate::sensors::retry::{Bme680Driver, Max44Driver, Sht31Driver};

pub mod fast;
pub mod slow;
pub mod retry;

// Todo make failed init not critical. Keep trying init in background
// while we are measuring
//
pub type I2cError = I2cDeviceError<embassy_stm32::i2c::Error>;
pub type UartError = embassy_stm32::usart::Error;

pub mod concrete_types {
    use embassy_embedded_hal::shared_bus;
    use embassy_stm32::i2c::I2c;
    use embassy_stm32::mode::Async;
    use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    use shared_bus::asynch::i2c::I2cDevice;

    pub type ConcreteSharedI2c<'a> = I2cDevice<'a, NoopRawMutex, I2c<'static, Async>>;
    pub type ConcreteTx<'a> = UartTx<'a, Async>;
    pub type ConcreteRx<'a> = RingBufferedUartRx<'a>;
}

pub async fn init_then_measure(
    publish: &Queues,
    i2c: Mutex<NoopRawMutex, I2c<'static, Async>>,
    usart_mhz: Uart<'static, Async>,
    usart_sps: Uart<'static, Async>,
) -> Result<(), Error> {
    info!("initializing sensors");
    let bme = Bme680Driver::new(&i2c);
    let max44009 = Max44Driver::new(&i2c);
    let sht = Sht31Driver::new(&i2c);

    let (tx, rx) = usart_mhz.split();
    let mut usart_buf = [0u8; 9 * 10]; // 9 byte messages
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let mhz = MHZ::from_tx_rx(tx, rx);

    let (tx, rx) = usart_sps.split();
    let mut usart_buf = [0u8; 100];
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let sps30 = with_timeout(Duration::from_millis(100), Sps30::from_tx_rx(tx, rx, Delay))
        .await
        .map_err(|_| Error::SetupTimedOut(Device::Sps30))?
        .map_err(SensorError::Sps30)
        .map_err(Error::Setup)?;

    let sensors_fast = fast::read(max44009, /*buttons,*/ &publish);
    let sensors_slow = slow::read(sht, bme, mhz, sps30, &publish);
    join::join(sensors_fast, sensors_slow).await;

    defmt::unreachable!();
}
