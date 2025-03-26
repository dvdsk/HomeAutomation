use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Instant, Timer};

use protocol::button;

pub async fn watch_button(
    mut input: ExtiInput<'static>,
    on_ok_press: impl AsyncFn(protocol::button::Press),
    on_err_press: impl Fn(),
) {
    let mut went_high_at: Option<Instant> = None;
    loop {
        if let Some(went_high_at) = went_high_at.take() {
            input.wait_for_low().await;
            let press = went_high_at.elapsed();
            let Ok(press) = press.as_millis().try_into().map(button::Press)
            else {
                on_err_press();
                continue;
            };
            on_ok_press(press).await;
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
