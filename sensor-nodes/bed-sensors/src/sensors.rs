use defmt::info;
use embassy_embedded_hal::shared_bus::I2cDeviceError;
use embassy_futures::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;
use mhzx::MHZ;
use protocol::large_bedroom::bed::Device;
use retry::Nau7802Driver;
use sps30_async::Sps30;

use crate::channel::Queues;
use crate::error_cache::{Error, SensorError};
use crate::sensors::retry::{Bme680Driver, Max44Driver, Sht31Driver, Sps30Driver};

use self::retry::Driver;

pub mod fast;
pub mod retry;
pub mod slow;

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

const SPS30_UART_BUF_SIZE: usize = 100;
const SPS30_DRIVER_BUF_SIZE: usize = 2 * SPS30_UART_BUF_SIZE;

pub async fn init_then_measure(
    publish: &Queues,
    i2c_1: Mutex<NoopRawMutex, I2c<'static, Async>>,
    i2c_3: Mutex<NoopRawMutex, I2c<'static, Async>>,
    usart_mhz: Uart<'static, Async>,
    usart_sps: Uart<'static, Async>,
) -> Result<(), Error> {
    info!("initializing sensors");
    let bme = Bme680Driver::new(&i2c_3, Device::Bme680);
    let max44009 = Max44Driver::new(&i2c_1, Device::Max44);
    let sht = Sht31Driver::new(&i2c_1, Device::Sht31);
    let nau_left = Nau7802Driver::new(&i2c_3, Device::Nau7802Left);

    let (tx, rx) = usart_mhz.split();
    let mut usart_buf = [0u8; 9 * 10]; // 9 byte messages
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let mhz = MHZ::from_tx_rx(tx, rx);

    let (tx, rx) = usart_sps.split();
    let mut usart_buf = [0u8; SPS30_UART_BUF_SIZE];
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let sps30 = Sps30Driver::init(tx, rx);

    let sensors_fast = fast::read(max44009, nau_left, /*buttons,*/ publish);
    let sensors_slow = slow::read(sht, bme, mhz, sps30, publish);
    join::join(sensors_fast, sensors_slow).await;

    defmt::unreachable!();
}

use concrete_types::ConcreteRx as Rx;
use concrete_types::ConcreteTx as Tx;
impl<'a> Driver for MHZ<Tx<'a>, Rx<'a>> {
    type Measurement = mhzx::Measurement;

    #[inline(always)]
    async fn try_measure(&mut self) -> Result<Self::Measurement, crate::error_cache::Error> {
        self.read_co2()
            .await
            .map_err(SensorError::Mhz14)
            .map_err(Error::Running)
    }

    #[inline(always)]
    fn device(&self) -> protocol::large_bedroom::bed::Device {
        Device::Mhz14
    }
}

impl<'a> Driver for Sps30<SPS30_DRIVER_BUF_SIZE, Tx<'a>, Rx<'a>, Delay> {
    type Measurement = sps30_async::Measurement;

    #[inline(always)]
    async fn try_measure(&mut self) -> Result<Self::Measurement, crate::error_cache::Error> {
        self.read_measurement()
            .await
            .map_err(SensorError::Sps30)
            .map_err(Error::Running)
    }

    #[inline(always)]
    fn device(&self) -> protocol::large_bedroom::bed::Device {
        Device::Sps30
    }
}
