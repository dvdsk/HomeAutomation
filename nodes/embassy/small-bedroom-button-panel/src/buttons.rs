use embassy_futures::join;
use embassy_stm32::exti::ExtiInput;
use fast::watch_button;

pub mod fast;

pub struct Inputs {
    pub bottom_left: ExtiInput<'static>,
    pub bottom_middle: ExtiInput<'static>,
    pub bottom_right: ExtiInput<'static>,
    pub top_left: ExtiInput<'static>,
    pub top_middle: ExtiInput<'static>,
    pub top_right: ExtiInput<'static>,
}

macro_rules! on_ok {
    ($pos:ident) => {
        |press| {
            crate::PUBLISH.send(protocol::Reading::SmallBedroom(
                protocol::small_bedroom::Reading::ButtonPanel(
                    protocol::small_bedroom::ButtonPanel::$pos(press),
                ),
            ))
        }
    };
}

macro_rules! on_err {
    ($pos:ident) => {
        || {
            let event_for_printing =
                (protocol::small_bedroom::ButtonPanel::$pos)(
                    protocol::button::Press(0),
                );
            let name = event_for_printing.variant_name();
            defmt::error!("Button pressed too long: {}", name);
        }
    };
}

pub async fn init_then_measure(inputs: Inputs) {
    join::join(
        join::join3(
            watch_button(
                inputs.bottom_left,
                on_ok!(BottomLeft),
                on_err!(BottomLeft),
            ),
            watch_button(
                inputs.bottom_middle,
                on_ok!(BottomMiddle),
                on_err!(BottomMiddle),
            ),
            watch_button(
                inputs.bottom_right,
                on_ok!(BottomRight),
                on_err!(BottomRight),
            ),
        ),
        join::join3(
            watch_button(inputs.top_left, on_ok!(TopLeft), on_err!(TopLeft)),
            watch_button(
                inputs.top_middle,
                on_ok!(TopMiddle),
                on_err!(TopMiddle),
            ),
            watch_button(inputs.top_right, on_ok!(TopRight), on_err!(TopRight)),
        ),
    )
    .await;
}
