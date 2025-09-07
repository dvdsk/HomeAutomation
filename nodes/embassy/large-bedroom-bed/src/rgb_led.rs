use embassy_executor::task;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::peripherals::{PB13, PB14, PB15, TIM1};
use embassy_stm32::time::khz;
use embassy_stm32::timer::complementary_pwm::{
    ComplementaryPwm, ComplementaryPwmPin,
};
use embassy_stm32::{timer, Peri};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

struct RgbLed {
    pwm: ComplementaryPwm<'static, TIM1>,
    r: f32,
    g: f32,
    b: f32,
}

impl RgbLed {
    fn new(
        timer: Peri<'static, TIM1>,
        b: Peri<'static, PB15>,
        r: Peri<'static, PB14>,
        g: Peri<'static, PB13>,
    ) -> Self {
        use embassy_stm32::timer::Channel;

        let b = ComplementaryPwmPin::new(b, OutputType::PushPull);
        let r = ComplementaryPwmPin::new(r, OutputType::PushPull);
        let g = ComplementaryPwmPin::new(g, OutputType::PushPull);

        let mut pwm = ComplementaryPwm::new(
            timer,
            None,
            Some(g),
            None,
            Some(r),
            None,
            Some(b),
            None,
            None,
            khz(10),
            Default::default(),
        );

        for channel in [Channel::Ch1, Channel::Ch2, Channel::Ch3] {
            pwm.set_duty(channel, pwm.get_max_duty());
            pwm.enable(channel);
        }

        Self {
            pwm,
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    fn set(&mut self, channel: timer::Channel, brightness: f32) {
        let max = self.pwm.get_max_duty();
        let scaled = (brightness * max as f32) as u16;
        let complementary_duty = max - scaled;
        self.pwm.set_duty(channel, complementary_duty);
    }

    fn set_red(&mut self, brightness: f32) {
        self.set(timer::Channel::Ch1, brightness);
    }

    fn set_blue(&mut self, brightness: f32) {
        self.set(timer::Channel::Ch2, brightness);
    }

    fn set_green(&mut self, brightness: f32) {
        self.set(timer::Channel::Ch3, brightness);
    }

    fn update(&mut self, bri: f32) {
        let sum = (self.r + self.b + self.g).min(1.0);
        let r = self.r / sum;
        let b = self.b / sum;
        let g = self.g / sum;
        self.set_red(bri * r);
        self.set_blue(bri * b);
        self.set_green(bri * g);
    }
}

pub(crate) enum Event {
    SetColor { r: f32, g: f32, b: f32 },
    LuxChanged(f32),
}

#[derive(Clone)]
pub struct LedHandle {
    led_comms: &'static Channel<ThreadModeRawMutex, Event, 5>,
}

impl LedHandle {
    pub const fn new(
        led_comms: &'static Channel<ThreadModeRawMutex, Event, 5>,
    ) -> Self {
        Self { led_comms }
    }
    pub async fn set_color(&self, r: f32, g: f32, b: f32) {
        self.led_comms.send(Event::SetColor { r, g, b }).await
    }

    pub async fn update_lux(&self, lux: f32) {
        self.led_comms.send(Event::LuxChanged(lux)).await
    }
}

fn lookup_bri(lux: f32) -> f32 {
    const LOOKUP_TABLE: [(f32, f32); 7] = [
        (5., 0.01),
        (10., 0.05),
        (20., 0.1),
        (40., 0.2),
        (80., 0.4),
        (160., 0.5),
        (320., 0.6),
    ];

    LOOKUP_TABLE
        .iter()
        .find(|(lux_entry, _)| *lux_entry > lux)
        .map(|(_, bri)| bri)
        .copied()
        .unwrap_or(0.6)
}

#[task]
pub(crate) async fn control_leds(
    timer: Peri<'static, TIM1>,
    r: Peri<'static, PB14>,
    g: Peri<'static, PB13>,
    b: Peri<'static, PB15>,
    led_comms: &'static Channel<ThreadModeRawMutex, Event, 5>,
) {
    let mut led = RgbLed::new(timer, b, r, g);
    let mut bri = 0.01; // Start with minimal bri

    loop {
        let order = led_comms.receive().await;
        match order {
            Event::SetColor { r, g, b } => {
                led.r = r;
                led.g = g;
                led.b = b;
            }
            Event::LuxChanged(new) => {
                bri = lookup_bri(new);
            }
        }

        led.update(bri);
    }
}
