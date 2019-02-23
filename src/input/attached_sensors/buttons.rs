extern crate sysfs_gpio;
extern crate futures;
extern crate tokio;

use futures::{Future, lazy, Stream};
use sysfs_gpio::{Direction, Edge, Pin};
use std::sync::mpsc::Sender;
use std::thread;

use crate::controller::{Command, Event};

fn stream(pin_nums: Vec<u64>, tx: Sender<Event>) -> sysfs_gpio::Result<()> {
    // NOTE: this currently runs forever and as such if
    // the app is stopped (Ctrl-C), no cleanup will happen
    // and the GPIO will be left exported.  Not much
    // can be done about this as Rust signal handling isn't
    // really present at the moment.  Revisit later.
    let pins: Vec<_> = pin_nums.iter().map(|&p| (p, Pin::new(p))).collect();
    let task = lazy(move || {
        for &(i, ref pin) in pins.iter() {
            pin.export().unwrap();
            pin.set_direction(Direction::In).unwrap();
            pin.set_edge(Edge::BothEdges).unwrap();
            let tx = tx.clone();
            tokio::spawn(pin.get_value_stream().unwrap()
                .for_each(move |val| {
                    println!("Pin {} changed value to {}", i, val);
                    tx.send(Event::Command(Command::LampsToggle));
                    Ok(())
                })
                .map_err(|_| ()));
        }
        Ok(())
    });
    tokio::run(task);
    Ok(())
}

pub fn start(tx: Sender<Event>) {
  thread::spawn(move || {
		let pins = vec!(22,23,24); //BCM pin number
		stream(pins, tx).unwrap();
	});
}
