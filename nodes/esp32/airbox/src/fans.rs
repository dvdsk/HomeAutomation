use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use esp_hal::gpio::GpioPin;
use esp_hal::ledc::channel::{self, Channel, ChannelIFace, Number};
use esp_hal::ledc::timer::{Timer, TimerIFace};
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, timer};
use esp_hal::peripherals::LEDC;
use esp_hal::time::Rate;
use protocol::large_bedroom::airbox::{DeviceError, Error};
use protocol::make_error_string;

use crate::wrap_error;

/// stack allocated parts of Fans
pub struct FanStack {
    control: Ledc<'static>,
    timer: Timer<'static, LowSpeed>,
}

pub struct Fans<'a> {
    fan_channel: Channel<'a, LowSpeed>,
    transition: Mutex<NoopRawMutex, Transition>,
}

struct Transition {
    transition_started: Instant,
    power_pre_transition: u8,
    power_target: u8,
}

impl Transition {
    fn fade_time(&self, target: u8) -> u16 {
        const RATE: u16 = 2000 / 100;
        let target_change = target as i8 - self.current_power() as i8;
        let target_change = target_change.abs() as u16;
        target_change * RATE
    }
    fn current_power(&self) -> u8 {
        let target_change =
            self.power_target as i8 - self.power_pre_transition as i8;
        let progress =
            self.transition_started.elapsed().as_millis() as f32 / 1000.0;
        let progress = if progress > 1.0 { 1.0 } else { progress };

        let curr = self.power_pre_transition as i8
            + (progress * target_change as f32) as i8;
        curr as u8
    }
    fn store_new_to(&mut self, target: u8) {
        self.power_pre_transition = self.current_power();
        self.power_target = target;
        self.transition_started = Instant::now();
    }
}

impl FanStack {
    pub fn new(led_controller: LEDC) -> Self {
        let mut control = Ledc::new(led_controller);
        control.set_global_slow_clock(LSGlobalClkSource::APBClk);
        let mut timer = control.timer(timer::Number::Timer0);
        defmt::unwrap!(timer.configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        }));
        FanStack { control, timer }
    }
}

impl<'a> Fans<'a> {
    pub fn new(stack: &'a FanStack, pwm_pin: GpioPin<3>) -> Self {
        let mut fan_channel = stack.control.channel(Number::Channel0, pwm_pin);
        defmt::unwrap!(fan_channel.configure(channel::config::Config {
            timer: &stack.timer,
            duty_pct: 10,
            pin_config: channel::config::PinConfig::PushPull,
        }));
        let transition = Mutex::new(Transition {
            transition_started: Instant::now(),
            power_pre_transition: 0,
            power_target: 0,
        });
        Self {
            fan_channel,
            transition,
        }
    }

    pub async fn current_power(&self) -> u8 {
        self.transition.lock().await.current_power()
    }

    pub async fn fade_time(&self, target: u8) -> u16 {
        self.transition.lock().await.fade_time(target)
    }

    pub async fn set_power(&self, target: u8) -> Result<(), protocol::Error> {
        if let Err(e) = self.fan_channel.start_duty_fade(
            self.current_power().await,
            target,
            self.fade_time(target).await,
        ) {
            return Err(wrap_error(Error::Running(DeviceError::Pwm(
                make_error_string(e),
            ))));
        }
        self.transition.lock().await.store_new_to(target);
        Ok(())
    }
}
