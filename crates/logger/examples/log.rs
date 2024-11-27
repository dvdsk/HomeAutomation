use std::thread;
use std::time::Duration;

use tracing::field::FieldSet;
use tracing::{Level, Metadata};
use tracing_core::callsite::DefaultCallsite;

// use logger::{ratelimited::Logger, warn};
//
// fn main() {
//     logger::tracing::setup();
//
//     let mut logger = Logger::default().with_per_msg_period(Duration::from_millis(800));
//
//     for i in 0..10 {
//         warn!(logger; "constant message always the same");
//         warn!(logger; "oh no the number is: {}", i);
//         thread::sleep(Duration::from_millis(100));
//     }
// }

fn main() {
    logger::tracing::setup();

    tracing::info!("works");
    let callsite = DefaultCallsite
    let fields = FieldSet::new(names, callsite)
    let meta = Metadata::new("test", "test", Level::INFO, None, None, None, FieldSet::new 
    tracing::Event::dispatch(DefaultCallsite::new, kind))


    static META: $crate::Metadata<'static> = {
    $crate::metadata! {
        name: $name,
        target: $target,
        level: $lvl,
        fields: $crate::fieldset!( $($fields)* ),
        callsite: &__CALLSITE,
        kind: $kind,
        }
    };
    $crate::callsite::DefaultCallsite::new(&META)
}
