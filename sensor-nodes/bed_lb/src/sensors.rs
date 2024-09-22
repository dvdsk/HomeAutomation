use defmt::info;
use embassy_embedded_hal::shared_bus::I2cDeviceError;
use embassy_futures::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;
use fast::ButtonInputs;
use mhzx::MHZ;
use protocol::large_bedroom::bed::Device;
use sps30_async::Sps30;

use crate::channel::Queues;
use crate::error_cache::{Error, SensorError};
use crate::rgb_led;

pub mod fast;
pub mod retry;
pub mod slow;

use retry::{
    Bme680Driver, Driver, Max44Driver, Nau7802Driver, Nau7802DriverBlocking, Sht31Driver,
    Sps30Driver,
};

pub type I2cError = I2cDeviceError<embassy_stm32::i2c::Error>;
pub type UartError = embassy_stm32::usart::Error;

pub mod concrete_types {
    use embassy_embedded_hal::shared_bus;
    use embassy_stm32::i2c::I2c;
    use embassy_stm32::mode::{Async, Blocking};
    use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    use shared_bus::asynch::i2c::I2cDevice;

    use super::I2cWrapper;

    pub type ConcreteSharedI2c<'a> = I2cDevice<'a, NoopRawMutex, I2c<'static, Async>>;
    pub type ConcreteBlockingI2c<'a> = &'a I2cWrapper<I2c<'static, Blocking>>;
    pub type ConcreteTx<'a> = UartTx<'a, Async>;
    pub type ConcreteRx<'a> = RingBufferedUartRx<'a>;
}

const SPS30_UART_BUF_SIZE: usize = 150;
const SPS30_DRIVER_BUF_SIZE: usize = 2 * SPS30_UART_BUF_SIZE;

pub async fn init_then_measure(
    publish: &Queues,
    orderers: &slow::DriverOrderers,
    rgb_led: rgb_led::LedHandle<'_>,
    i2c_1: Mutex<NoopRawMutex, I2c<'static, Async>>,
    i2c_2: Mutex<NoopRawMutex, I2c<'static, Blocking>>,
    i2c_3: Mutex<NoopRawMutex, I2c<'static, Async>>,
    usart_mhz: Uart<'static, Async>,
    usart_sps: Uart<'static, Async>,
    buttons: ButtonInputs,
) -> Result<(), Error> {
    info!("initializing sensors");
    let bme = Bme680Driver::new(&i2c_3, Device::Bme680);
    let max44009 = Max44Driver::new(&i2c_1, Device::Max44);
    let sht = Sht31Driver::new(&i2c_1, Device::Sht31);
    let i2c_2 = I2cWrapper(i2c_2);
    let nau_right = Nau7802DriverBlocking::new(&i2c_2, Device::Nau7802Right);
    let nau_left = Nau7802Driver::new(&i2c_3, Device::Nau7802Left);

    let (tx, rx) = usart_mhz.split();
    let mut usart_buf = [0u8; 9 * 10]; // 9 byte messages
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let mhz = MHZ::from_tx_rx(tx, rx);

    let (tx, rx) = usart_sps.split();
    let mut usart_buf = [0u8; SPS30_UART_BUF_SIZE];
    let rx = rx.into_ring_buffered(&mut usart_buf);
    let sps30 = Sps30Driver::init(tx, rx);

    let sensors_fast = fast::read(max44009, nau_right, nau_left, buttons, publish, rgb_led);
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

use concrete_types::ConcreteRx as Rx;
use concrete_types::ConcreteTx as Tx;
impl<'a> Driver for MHZ<Tx<'a>, Rx<'a>> {
    type Measurement = mhzx::Measurement;
    type Affector = ();

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
    type Affector = ();

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

pub(crate) struct I2cWrapper<T>(Mutex<NoopRawMutex, T>);

impl<T: embedded_hal::i2c::ErrorType> embedded_hal_async::i2c::ErrorType for &I2cWrapper<T> {
    type Error = <T as embedded_hal::i2c::ErrorType>::Error;
}

impl<T: embedded_hal::i2c::I2c> embedded_hal_async::i2c::I2c for &I2cWrapper<T> {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.0.lock().await.transaction(address, operations)
    }

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.0.lock().await.read(address, read)
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.0.lock().await.write(address, write)
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0.lock().await.write_read(address, write, read)
    }
}
