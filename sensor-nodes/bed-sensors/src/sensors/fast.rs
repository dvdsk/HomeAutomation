use defmt::warn;
use embassy_futures::join::{join3, join4};
use embassy_futures::{join, yield_now};
use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Instant, Timer};

use protocol::large_bedroom::{bed::Button, bed::Reading};

use super::retry::{Max44Driver, Nau7802Driver, Nau7802DriverBlocking};
use crate::channel::Queues;
use crate::rgb_led;

fn sig_lux_diff(old: f32, new: f32) -> bool {
    let diff = old - new;
    // we do not have f32::abs on embedded
    diff > old / 20.0 || -diff > old / 20.0
}

fn sig_weight_diff(old: u32, new: u32) -> bool {
    let diff = new.abs_diff(old);
    diff > 2000
}

async fn report_lux(mut max44: Max44Driver<'_>, publish: &Queues, rgb_led: rgb_led::LedHandle<'_>) {
    const MAX_INTERVAL: Duration = Duration::from_secs(5);

    let mut prev_lux = f32::MAX;
    let mut last_lux = Instant::now();

    // todo!("reinit devices after error");
    loop {
        Timer::after_millis(50).await;
        let lux = match max44.try_measure().await {
            Ok(lux) => lux,
            Err(err) if last_lux.elapsed() > MAX_INTERVAL => {
                publish.queue_error(err);
                continue;
            }
            Err(_) => continue,
        };

        if sig_lux_diff(prev_lux, lux) {
            publish.send_p2(Reading::Brightness(lux));
            rgb_led.update_lux(lux).await;
        } else if last_lux.elapsed() > MAX_INTERVAL {
            publish.send_p1(Reading::Brightness(lux));
        } else {
            yield_now().await;
            continue;
        };

        prev_lux = lux;
        last_lux = Instant::now();
    }
}

trait Nau {
    async fn try_measure(&mut self) -> Result<u32, crate::error_cache::Error>;
}

impl Nau for Nau7802Driver<'_> {
    async fn try_measure(&mut self) -> Result<u32, crate::error_cache::Error> {
        Nau7802Driver::try_measure(self).await
    }
}

impl Nau for Nau7802DriverBlocking<'_> {
    async fn try_measure(&mut self) -> Result<u32, crate::error_cache::Error> {
        Nau7802DriverBlocking::try_measure(self).await
    }
}

async fn report_weight(mut nau: impl Nau, wrap: impl Fn(u32) -> Reading, publish: &Queues) {
    const MAX_INTERVAL: Duration = Duration::from_secs(5);

    let mut prev_weight = u32::MAX;
    let mut reported_at = Instant::now();

    loop {
        Timer::after_millis(100).await;
        let weight = match nau.try_measure().await {
            Ok(lux) => lux,
            Err(err) if reported_at.elapsed() > MAX_INTERVAL => {
                publish.queue_error(err);
                continue;
            }
            Err(_) => continue,
        };

        if sig_weight_diff(prev_weight, weight) {
            publish.send_p2(wrap(weight));
        } else if reported_at.elapsed() > MAX_INTERVAL {
            publish.send_p1(wrap(weight));
        } else {
            yield_now().await;
            continue;
        };

        prev_weight = weight;
        reported_at = Instant::now();
    }
}

async fn watch_button(
    mut input: ExtiInput<'static>,
    event: impl Fn(protocol::button::Press) -> Button,
    channel: &Queues,
) {
    let mut went_high_at: Option<Instant> = None;
    loop {
        if let Some(went_high_at) = went_high_at.take() {
            input.wait_for_low().await;
            let press = went_high_at.elapsed();
            let Ok(press) = press.as_millis().try_into() else {
                warn!("extremely long button press registered, skipping");
                continue;
            };
            let event = (event)(protocol::button::Press(press));
            channel.send_p2(Reading::Button(event));
        } else {
            input.wait_for_rising_edge().await;
            Timer::after(Duration::from_millis(50)).await;
            if input.is_high() {
                went_high_at = Some(Instant::now());
            }
        }
    }
}

pub struct ButtonInputs {
    pub top: ExtiInput<'static>,
    pub middle_inner: ExtiInput<'static>,
    pub middle_center: ExtiInput<'static>,
    pub middle_outer: ExtiInput<'static>,
    pub lower_inner: ExtiInput<'static>,
    pub lower_center: ExtiInput<'static>,
    pub lower_outer: ExtiInput<'static>,
}

pub(crate) async fn read(
    max44: Max44Driver<'_>,
    nau_right: Nau7802DriverBlocking<'_>,
    nau_left: Nau7802Driver<'_>,
    inputs: ButtonInputs,
    publish: &Queues,
    rgb_led: rgb_led::LedHandle<'_>,
) {
    use protocol::large_bedroom::bed::Button;

    let watch_buttons_1 = join4(
        watch_button(inputs.top, Button::Top, publish),
        watch_button(inputs.middle_inner, Button::MiddleInner, publish),
        watch_button(inputs.middle_center, Button::MiddleCenter, publish),
        watch_button(inputs.middle_outer, Button::MiddleOuter, publish),
    );

    let watch_buttons_2 = join3(
        watch_button(inputs.lower_inner, Button::LowerInner, publish),
        watch_button(inputs.lower_center, Button::LowerCenter, publish),
        watch_button(inputs.lower_outer, Button::LowerOuter, publish),
    );

    let watch_buttons = join::join(watch_buttons_1, watch_buttons_2);

    let watch_lux = report_lux(max44, publish, rgb_led);
    let report_right_weight = report_weight(nau_right, Reading::WeightRight, publish);
    let report_left_weight = report_weight(nau_left, Reading::WeightLeft, publish);

    join::join4(
        report_left_weight,
        report_right_weight,
        watch_lux,
        watch_buttons,
    )
    .await;
}
