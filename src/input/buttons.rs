extern crate sysfs_gpio;
extern crate futures;
extern crate tokio;

use futures::{Future, lazy, Stream};
use sysfs_gpio::{Direction, Edge, Pin};
use std::thread;
use std::time::{Duration, Instant};
use crossbeam_channel;

use crate::controller::{Command, Event};

// when pressing 22, 24 is often activated
// 23 works perfectly
// 24 also works perfectly

fn stream(mut pin_numb_command_pairs: Vec<(u64, Command)>, tx: crossbeam_channel::Sender<Event>) -> sysfs_gpio::Result<()> {
    // NOTE: this currently runs forever and as such if
    // the app is stopped (Ctrl-C), no cleanup will happen
    // and the GPIO will be left exported.  Not much
    // can be done about this as Rust signal handling isn't
    // really present at the moment.  Revisit later.
    let task = lazy(move || {
        for (pin_numb, command) in pin_numb_command_pairs.drain(..) {
        	let mut last_press = Instant::now();
        	let pin = Pin::new(pin_numb);
            pin.export().expect("could not export pin");
            pin.set_direction(Direction::In).expect("could not set direction for pin");
            pin.set_edge(Edge::RisingEdge).expect("could not set interrupt to rising edge for pin");
            let tx = tx.clone();
            tokio::spawn(pin.get_value_stream().unwrap()
                .for_each(move |val| { //TODO refactor
                		if val == 1 {
		              		if last_press.elapsed() > Duration::from_millis(50) {
		              			last_press = Instant::now();
		              			dbg!(pin_numb);
		              			dbg!(val);
		                  	tx.send(Event::Command(command.clone())).unwrap();
		                  }
                    }
                    Ok(())
                })
                .map_err(|_| ()));
        }
        Ok(())
    });
    tokio::run(task);
    Ok(())
}

pub fn start_monitoring(tx: crossbeam_channel::Sender<Event>) {
  thread::spawn(move || {
		let pin_numb_command_pairs = vec!(
            (16, Command::LampsDim),
            (12, Command::LampsDimmest),
            (13, Command::LampsToggle),
            
            (27, Command::LampsToggle), //left 3, left
            (22, Command::LampsToggle), //left 3, middle
            (18, Command::LampsToggle), //left 3, right
            
            (23, Command::LampsNight), //right 4, left most
            (24, Command::LampsEvening), //right 4, left 
            (26, Command::LampsDay), //right 4, right
            (17, Command::LampsToggle), //right 4, right most
        ); //BCM pin number
		
		stream(pin_numb_command_pairs, tx).unwrap();
	});
}
