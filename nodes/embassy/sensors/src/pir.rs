use embassy_stm32::exti::ExtiInput;
use embassy_time::{Timer, Duration};

use protocol::pir::Status;

pub async fn watch_array<const N: usize, P, F>(
    pirs: [(ExtiInput<'static>, F); N],
    on_pir: impl Fn(P) + Clone,
) where
    P: defmt::Format,
    F: Fn(protocol::pir::Status) -> P + Clone,
{
    embassy_futures::join::join_array(pirs.map(|(input, into_pir)| {
        let local_into_pir = into_pir.clone();
        let local_on_pir = on_pir.clone();
        let on_ok = move |press| {
            let on_pir = &local_on_pir;
            let into_pir = &local_into_pir;
            on_pir(into_pir(press));
        };

        watch_pir(input, on_ok)
    }))
    .await;
}

async fn watch_pir(
    mut input: ExtiInput<'static>,
    on_change: impl Fn(protocol::pir::Status),
) {
    loop {
        input.wait_for_high().await;
        Timer::after(Duration::from_millis(25)).await;
        if input.is_low() {
            continue;
        }
        Timer::after(Duration::from_millis(25)).await;
        if input.is_low() {
            continue;
        }
        Timer::after(Duration::from_millis(25)).await;
        if input.is_low() {
            continue;
        }
        // pretty sure at this point this is not some random noise
        on_change(Status::Started);
        loop {
            input.wait_for_low().await;
            Timer::after(Duration::from_millis(25)).await;
            if input.is_high() {
                continue;
            }
            Timer::after(Duration::from_millis(25)).await;
            if input.is_high() {
                continue;
            }
            Timer::after(Duration::from_millis(25)).await;
            if input.is_high() {
                continue;
            }
            // pretty sure at this point this is not some random noise
            on_change(Status::End);
            break;
        }
    }
}
