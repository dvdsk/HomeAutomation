use embedded_hal_async::digital::Wait;
use protocol::{Press, Reading};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace};

use std::thread;

use protocol::large_bedroom::desk::{Button, SetupError};

use crate::{send_error, send_reading};

const CHIP: &str = "/dev/gpiochip0";

async fn watch_pin(
    offset: u32,
    as_button: impl Fn(Press) -> Button,
    tx: &Sender<Result<Reading, protocol::Error>>,
) {
    use protocol::large_bedroom::desk;

    const DEBOUNCE: Duration = Duration::from_millis(5);
    const MAX_PRESS: Duration = Duration::from_secs(5);

    // pins are pulled down to ground
    let mut pin = match gpiocdev_embedded_hal::tokio::InputPin::new(CHIP, offset) {
        Ok(pin) => pin,
        Err(error) => {
            error!("error opening gpio {offset} on {CHIP}: {error}");
            let error: protocol::downcast_err::GpioError = error.into();
            let error = desk::Error::Setup(SetupError::Gpio(error));
            send_error(tx, error);
            return;
        }
    };

    info!("opened_pin: {offset}");

    loop {
        if let Err(error) = pin.wait_for_rising_edge().await {
            error!("error waiting for rising edge for gpio {offset} on {CHIP}: {error}");
            let error: protocol::downcast_err::GpioError = error.into();
            let error = desk::Error::Running(desk::SensorError::Gpio(error));
            send_error(tx, error);
            return;
        }

        let now = Instant::now();
        trace!("waiting for button to be released");
        match tokio::time::timeout(MAX_PRESS, pin.wait_for_falling_edge()).await {
            Ok(Err(error)) => {
                error!("error waiting for falling edge for gpio {offset} on {CHIP}: {error}");
                let error: protocol::downcast_err::GpioError = error.into();
                let error = desk::Error::Running(desk::SensorError::Gpio(error));
                send_error(tx, error);
                return;
            }
            Ok(Ok(())) => (),
            Err(_timeout) => continue,
        }

        let press = now.elapsed();
        if press > DEBOUNCE {
            debug!("sending button press (pressed for: {press:?})");
            let button = (as_button)(Press(press.as_millis() as u16));
            let reading = desk::Reading::Button(button);
            send_reading(tx, reading);
        } else {
            trace!("press too short, caught as bounce");
        }
    }
}

async fn watch_pins(tx: &Sender<Result<Reading, protocol::Error>>) {
    tokio::join!(
        watch_pin(27, Button::OneOfThree, tx),
        watch_pin(22, Button::TwoOfThree, tx),
        watch_pin(18, Button::ThreeOfThree, tx),
        watch_pin(23, Button::FourOfFour, tx),
        watch_pin(24, Button::ThreeOfFour, tx),
        watch_pin(26, Button::TwoOfFour, tx),
        watch_pin(17, Button::OneOfFour, tx),
    );
    unreachable!("none of those should return, and never all of them");
}

pub fn start_monitoring(tx: Sender<Result<Reading, protocol::Error>>) {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        rt.block_on(watch_pins(&tx))
    });
}
