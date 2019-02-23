extern crate sysfs_gpio;
extern crate futures;
extern crate tokio;

use futures::{Future, lazy, Stream};
use sysfs_gpio::{Direction, Edge, Pin};
use std::sync::mpsc::Sender;
use std::thread;

use crate::controller::{Command, Event};

fn stream(mut pin_numb_command_pairs: Vec<(u64, Command)>, tx: Sender<Event>) -> sysfs_gpio::Result<()> {
    // NOTE: this currently runs forever and as such if
    // the app is stopped (Ctrl-C), no cleanup will happen
    // and the GPIO will be left exported.  Not much
    // can be done about this as Rust signal handling isn't
    // really present at the moment.  Revisit later.
    let task = lazy(move || {
        for (pin_numb, command) in pin_numb_command_pairs.drain(..) {
        		let pin = Pin::new(pin_numb);
            pin.export().unwrap();
            pin.set_direction(Direction::In).unwrap();
            pin.set_edge(Edge::RisingEdge).unwrap();
            let tx = tx.clone();
            tokio::spawn(pin.get_value_stream().unwrap()
                .for_each(move |val| {
                		dbg!(pin_numb);
                    tx.send(Event::Command(command.clone()));
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
		let pin_numb_command_pairs = vec!((22, Command::LampsDim),
		                                  (23, Command::LampsDimmest),
		                                  (24, Command::LampsToggle)); //BCM pin number
		stream(pin_numb_command_pairs, tx).unwrap();
	});
}
