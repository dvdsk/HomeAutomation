mod reinit_on_error;

mod driver_impl_bme680;
mod driver_impl_max44009;
mod driver_impl_nau7802;
mod driver_impl_sht31;
mod wrap_mhz;
mod wrap_sps30;

use reinit_on_error::ReInitOnErrorDriver;

use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use shared_bus::asynch::i2c::I2cDevice;

pub type ConcreteSharedI2c<'a> =
    I2cDevice<'a, NoopRawMutex, I2c<'static, Async>>;
pub type ConcreteTx<'a> = UartTx<'a, Async>;
pub type ConcreteRx<'a> = RingBufferedUartRx<'a>;

use bosch_bme680::Bme680;
use embassy_time::Delay;
use max44009::Max44009;
use nau7802_async::Nau7802;
use sht31::mode::SingleShot;
use sht31::SHT31;

// Driver type aliases/wrapper structs. These auto correct/re-init on error
pub use wrap_mhz::MhzDriver;
pub use wrap_sps30::{Sps30Driver, SPS30_DRIVER_BUF_SIZE, SPS30_UART_BUF_SIZE};
pub type Nau7802Driver<'a> =
    ReInitOnErrorDriver<Nau7802<ConcreteSharedI2c<'a>, Delay>>;
pub type Max44Driver<'a> = ReInitOnErrorDriver<Max44009<ConcreteSharedI2c<'a>>>;
pub type Bme680Driver<'a> =
    ReInitOnErrorDriver<Bme680<ConcreteSharedI2c<'a>, Delay>>;
pub type Sht31Driver<'a> =
    ReInitOnErrorDriver<SHT31<SingleShot, ConcreteSharedI2c<'a>>>;
