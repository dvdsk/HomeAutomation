use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Instant, Timer};
use protocol::button;

pub async fn watch_array<const N: usize, B, F>(
    buttons: [(ExtiInput<'static>, F); N],
    on_button: impl Fn(B) + Clone,
) where
    B: defmt::Format,
    F: Fn(protocol::button::Press) -> B + Clone,
{
    embassy_futures::join::join_array(buttons.map(|(input, into_button)| {
        let local_into_button = into_button.clone();
        let on_err = move || print_error(&local_into_button);

        let local_into_button = into_button.clone();
        let local_on_button = on_button.clone();
        let on_ok = move |press| {
            let on_button = &local_on_button;
            let into_button = &local_into_button;
            on_button(into_button(press));
        };

        watch_button(input, on_ok, on_err)
    }))
    .await;
}

fn print_error<B: defmt::Format>(
    into_reading: impl Fn(protocol::button::Press) -> B,
) {
    let event_for_printing = into_reading(protocol::button::Press(0));
    let name = event_for_printing; // todo variant trait
    defmt::error!("Button pressed too long: {:?}", name);
}

async fn watch_button(
    mut input: ExtiInput<'static>,
    on_ok_press: impl Fn(protocol::button::Press),
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
            on_ok_press(press);
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
