use defmt::warn;
use embassy_futures::{join, yield_now};
use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Instant, Timer};

use protocol::large_bedroom::{bed::Button, bed::Reading};

use super::retry::{Max44Driver, Nau7802Driver};
use crate::channel::Queues;

fn sig_lux_diff(old: f32, new: f32) -> bool {
    let diff = old - new;
    // we do not have f32::abs on embedded
    diff > old / 20.0 || -diff > old / 20.0
}

fn sig_weight_diff(old: u32, new: u32) -> bool {
    let diff = new.abs_diff(old);
    diff > 20
}

async fn report_lux(mut max44: Max44Driver<'_>, publish: &Queues) {
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

async fn report_weight(mut nau: Nau7802Driver<'_>, publish: &Queues) {
    const MAX_INTERVAL: Duration = Duration::from_secs(5);

    let mut prev_weight = u32::MAX;
    let mut last_lux = Instant::now();

    // todo!("reinit devices after error");
    loop {
        Timer::after_millis(100).await;
        let weight = match nau.try_measure().await {
            Ok(lux) => lux,
            Err(err) if last_lux.elapsed() > MAX_INTERVAL => {
                publish.queue_error(err);
                continue;
            }
            Err(_) => continue,
        };

        if sig_weight_diff(prev_weight, weight) {
            publish.send_p2(Reading::WeightLeft(weight));
        } else if last_lux.elapsed() > MAX_INTERVAL {
            publish.send_p1(Reading::WeightLeft(weight));
        } else {
            yield_now().await;
            continue;
        };

        prev_weight = weight;
        last_lux = Instant::now();
    }
}

#[allow(dead_code)]
async fn watch_button(
    mut input: ExtiInput<'static>,
    event: impl Fn(protocol::button::Press) -> Button,
    channel: &Queues,
) {
    let mut went_high_at: Option<Instant> = None;
    loop {
        if let Some(went_high_at) = went_high_at.take() {
            input.wait_for_falling_edge().await;
            let press = went_high_at.elapsed();
            if press > Duration::from_millis(5) {
                let Ok(press) = press.as_millis().try_into() else {
                    warn!("extremely long button press registered, skipping");
                    continue;
                };
                let event = (event)(protocol::button::Press(press));
                channel.send_p2(Reading::Button(event));
            }
        } else {
            input.wait_for_rising_edge().await;
            went_high_at = Some(Instant::now());
        }
    }
}

#[allow(dead_code)]
pub struct ButtonInputs {
    pub top_left: ExtiInput<'static>,
    pub top_right: ExtiInput<'static>,
    pub middle_inner: ExtiInput<'static>,
    pub middle_center: ExtiInput<'static>,
    pub middle_outer: ExtiInput<'static>,
    pub lower_inner: ExtiInput<'static>,
    pub lower_center: ExtiInput<'static>,
    pub lower_outer: ExtiInput<'static>,
}

pub async fn read(
    max44: Max44Driver<'_>,
    nau_left: Nau7802Driver<'_>,
    /*inputs: ButtonInputs,*/ publish: &Queues,
) {
    // let watch_buttons_1 = join5(
    //     watch_button(inputs.top_left, BedButton::TopLeft, publish),
    //     watch_button(inputs.top_right, BedButton::TopRight, publish),
    //     watch_button(inputs.middle_inner, BedButton::MiddleInner, publish),
    //     watch_button(inputs.middle_center, BedButton::MiddleCenter, publish),
    //     watch_button(inputs.middle_outer, BedButton::MiddleOuter, publish),
    // );
    //
    // let watch_buttons_2 = join3(
    //     watch_button(inputs.lower_inner, BedButton::LowerInner, publish),
    //     watch_button(inputs.lower_center, BedButton::LowerCenter, publish),
    //     watch_button(inputs.lower_outer, BedButton::LowerOuter, publish),
    // );

    let watch_lux = report_lux(max44, publish);
    let report_weight = report_weight(nau_left, publish);
    join::join(watch_lux, report_weight).await;
    // join::join3(watch_buttons_1, watch_buttons_2, watch_lux).await;
}
