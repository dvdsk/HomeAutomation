use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use esp_hal::gpio::GpioPin;
use esp_hal::ledc::channel::{Channel, ChannelIFace, Number};
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed};
use esp_hal::peripherals::LEDC;
use protocol::large_bedroom::airbox::{DeviceError, Error};
use protocol::make_error_string;

use crate::wrap_error;

pub struct Fans {
    fan_channel: Channel<'static, LowSpeed>,
    _control: Ledc<'static>,
    transition: Mutex<NoopRawMutex, Transition>,
}

struct Transition {
    transition_started: Instant,
    power_pre_transition: u8,
    power_target: u8,
}

impl Transition {
    fn current_power(&self) -> u8 {
        let target_change =
            self.power_pre_transition as i8 - self.power_target as i8;
        let progress =
            self.transition_started.elapsed().as_millis() as f32 / 1000.0;
        let progress = if progress > 1.0 { 1.0 } else { progress };

        self.power_pre_transition + (progress * target_change as f32) as u8
    }
    fn store_new_to(&mut self, target: u8) {
        self.power_pre_transition = self.current_power();
        self.power_target = target;
        self.transition_started = Instant::now();
    }
}

impl Fans {
    pub fn new(led_controller: LEDC, pwm_pin: GpioPin<3>) -> Self {
        let mut control = Ledc::new(led_controller);
        control.set_global_slow_clock(LSGlobalClkSource::APBClk);
        let fan_channel = control.channel(Number::Channel0, pwm_pin);
        let transition = Mutex::new(Transition {
            transition_started: Instant::now(),
            power_pre_transition: 0,
            power_target: 0,
        });
        Self {
            _control: control,
            fan_channel,
            transition,
        }
    }

    pub async fn current_power(&self) -> u8 {
        self.transition.lock().await.current_power()
    }

    pub async fn set_power(&self, power: u8) -> Result<(), protocol::Error> {
        if let Err(e) = self.fan_channel.start_duty_fade(
            self.current_power().await,
            power,
            1000,
        ) {
            return Err(wrap_error(Error::Running(DeviceError::Pwm(
                make_error_string(e),
            ))));
        }
        self.transition.lock().await.store_new_to(power);
        Ok(())
    }
}
