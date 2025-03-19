use embassy_futures::join::join4;
use embassy_futures::{join, yield_now};
use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Instant, Timer};

use protocol::small_bedroom::{bed::Button, bed::Reading};

use crate::channel::Queues;
use sensors::{Max44Driver, Nau7802Driver};

fn sig_lux_diff(old: f32, new: f32) -> bool {
    let diff = old - new;
    // we do not have f32::abs on embedded
    diff > old / 20.0 || -diff > old / 20.0
}

fn sig_weight_diff(old: u32, new: u32) -> bool {
    let diff = new.abs_diff(old);
    diff > 2000
}

async fn report_lux(mut max44: Max44Driver<'_>, publish: &Queues) {
    const MAX_INTERVAL: Duration = Duration::from_secs(5);

    let mut prev_lux = f32::MAX;
    let mut last_lux = Instant::now();

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

trait Nau {
    async fn try_measure(&mut self) -> Result<u32, sensors::Error>;
}

impl Nau for Nau7802Driver<'_> {
    async fn try_measure(&mut self) -> Result<u32, sensors::Error> {
        Nau7802Driver::try_measure(self).await
    }
}

async fn report_weight(
    mut nau: impl Nau,
    wrap: impl Fn(u32) -> Reading,
    publish: &Queues,
) {
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
    use sensors::{errors::PressTooLong, errors::SensorError, Error};

    let mut went_high_at: Option<Instant> = None;
    loop {
        if let Some(went_high_at) = went_high_at.take() {
            input.wait_for_low().await;
            let press = went_high_at.elapsed();
            let Ok(press) = press.as_millis().try_into() else {
                let event_for_printing = (event)(protocol::button::Press(0));
                let name = event_for_printing.variant_name();
                channel.queue_error(Error::Running(SensorError::Button(
                    PressTooLong { button: name },
                )));
                continue;
            };
            let event = (event)(protocol::button::Press(press));
            channel.send_p2(Reading::Button(event));
        } else {
            input.wait_for_high().await;
            Timer::after(Duration::from_millis(20)).await;
            if input.is_low() {
                continue;
            }
            Timer::after(Duration::from_millis(20)).await;
            if input.is_low() {
                continue;
            }
            Timer::after(Duration::from_millis(10)).await;
            if input.is_high() {
                went_high_at = Some(Instant::now());
            }
        }
    }
}

pub struct ButtonInputs {
    pub left: ExtiInput<'static>,
    pub left_middle: ExtiInput<'static>,
    pub right_middle: ExtiInput<'static>,
    pub right: ExtiInput<'static>,
}

pub(crate) async fn read(
    max44: Max44Driver<'_>,
    nau: Nau7802Driver<'_>,
    inputs: ButtonInputs,
    publish: &Queues,
) {
    use protocol::small_bedroom::bed::Button;

    let watch_buttons = join4(
        watch_button(inputs.left, Button::Left, publish),
        watch_button(inputs.left_middle, Button::LeftMiddle, publish),
        watch_button(inputs.right_middle, Button::RightMiddle, publish),
        watch_button(inputs.right, Button::Right, publish),
    );

    let watch_lux = report_lux(max44, publish);
    let report_weight = report_weight(nau, Reading::Weight, publish);

    join::join3(report_weight, watch_lux, watch_buttons).await;
}
