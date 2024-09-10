use embassy_stm32::gpio::OutputType;
use embassy_stm32::peripherals::{PB13, PB14, PB15, TIM1};
use embassy_stm32::time::khz;
use embassy_stm32::timer;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;

struct RgbLed {
    pwm: ComplementaryPwm<'static, TIM1>,
    r: f32,
    g: f32,
    b: f32,
}

impl RgbLed {
    fn new(timer: TIM1, b: PB15, r: PB14, g: PB13) -> Self {
        use embassy_stm32::timer::Channel;

        let b = ComplementaryPwmPin::new_ch3(b, OutputType::PushPull);
        let r = ComplementaryPwmPin::new_ch2(r, OutputType::PushPull);
        let g = ComplementaryPwmPin::new_ch1(g, OutputType::PushPull);

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
        defmt::debug!("bri: {}, r: {}, g: {}, b: {}", bri, r, b, g);
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
pub struct LedHandle<'a> {
    sender: &'a Channel<NoopRawMutex, Event, 5>,
}

impl<'a> LedHandle<'a> {
    pub async fn set_color(&self, r: f32, g: f32, b: f32) {
        self.sender.send(Event::SetColor { r, g, b }).await
    }

    pub async fn update_lux(&self, lux: f32) {
        self.sender.send(Event::LuxChanged(lux)).await
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

pub(crate) struct LedController<'a> {
    orders: &'a Channel<NoopRawMutex, Event, 5>,
    led: RgbLed,
}

impl<'a> LedController<'a> {
    pub(crate) async fn control(&mut self) {
        let mut bri = 0.01; // Start with minimal bri

        loop {
            let order = self.orders.receive().await;
            match order {
                Event::SetColor { r, g, b } => {
                    self.led.r = r;
                    self.led.g = g;
                    self.led.b = b;
                }
                Event::LuxChanged(new) => {
                    bri = lookup_bri(new);
                }
            }

            self.led.update(bri);
        }
    }
}

pub fn controller_and_handle<'a>(
    timer: TIM1,
    r: PB14,
    g: PB13,
    b: PB15,
    channel: &'a Channel<NoopRawMutex, Event, 5>,
) -> (LedController<'a>, LedHandle<'a>) {
    let led = RgbLed::new(timer, b, r, g);
    (
        LedController {
            led,
            orders: channel,
        },
        LedHandle { sender: channel },
    )
}
