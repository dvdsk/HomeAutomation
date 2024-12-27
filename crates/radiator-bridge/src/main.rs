use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use clap::Parser;
use data_server::api::data_source;
use data_server::api::subscriber;
use data_server::api::subscriber::SubMessage as M;
use futures::FutureExt;
use futures_concurrency::future::Race;
use tracing::warn;
use zigbee_bridge::Controller;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Opt {
    /// IP address where to subscribe for updates
    #[clap(long)]
    data_server_subscribe: SocketAddr,

    /// IP address where to send readings
    #[clap(long)]
    data_server_update: SocketAddr,

    /// IP address for MQTT broker
    #[clap(long)]
    mqtt_ip: IpAddr,
}

enum RelevantMsg {
    SmallBedroom(f64),
    LargeBedroom(f64),
}

#[derive(Debug, Default)]
struct RadiatorState {
    last_set_at: Option<Instant>,
    last_set_value: Option<f64>,
}

impl RadiatorState {
    // You *must* set the `External_measured_room_sensor` property *at least*
    // every 3 hours. After 3 hours the TRV disables this function and resets
    // the value of the `External_measured_room_sensor` property to -8000
    // (disabled).
    //
    // You *should* set the `External_measured_room_sensor`
    // property *at most* every 30 minutes or every 0.1°C change in measured
    // room temperature.
    //
    // If `Radiator_covered` is `true` (Room Sensor Mode):
    // You *must* set the `External_measured_room_sensor` property *at least*
    // every 30 minutes. After 35 minutes the TRV disables this function and
    // resets the value of the `External_measured_room_sensor` property to
    // -8000 (disabled).
    //
    // You *should* set the `External_measured_room_sensor`
    // property *at most* every 5 minutes or every 0.1°C change in measured
    // room temperature. The unit of this value is 0.01 `°C` (so e.g. 21°C
    // would be represented as 2100).
    fn should_update_given(&mut self, temp: f64) -> bool {
        const T30MINUTES: Duration = Duration::from_secs(60 * 30);

        if self
            .last_set_value
            .is_none_or(|val| (val - temp).abs() >= 0.1)
            || self.last_set_at.is_none_or(|at| at.elapsed() > T30MINUTES)
        {
            self.last_set_at = Some(Instant::now());
            self.last_set_value = Some(temp);
            true
        } else {
            false
        }
    }
}

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() {
    let args = Opt::parse();
    logger::tracing::setup();

    let mut data_subscriber = subscriber::ReconnectingClient::new(
        args.data_server_subscribe,
        env!("CARGO_PKG_NAME").to_owned(),
    )
    .subscribe();

    let mut data_source = data_source::reconnecting::Client::new(
        args.data_server_update,
        Vec::new(),
        None,
    )
    .await
    .expect("address is correct");

    let (reading_tx, mut reading_rx) = tokio::sync::mpsc::channel(1024);
    let callback = move |reading| {
        reading_tx
            .try_send(reading)
            .expect("reading_rx should keep up and never drop")
    };
    let controller = Controller::start_bridge_with_reading_callback(
        args.mqtt_ip,
        "temp-bridge",
        callback,
    );

    let mut radiators = Radiators {
        controller,
        small_bedroom: RadiatorState::default(),
        large_bedroom: RadiatorState::default(),
    };

    loop {
        let get_reading = reading_rx
            .recv()
            .map(|r| r.expect("reading_tx should not drop"))
            .map(Event::ZigbeeReading);
        let get_msg = data_subscriber.next().map(Event::DataServerMsg);
        let event = (get_reading, get_msg).race().await;

        match event {
            Event::ZigbeeReading(reading) => {
                if let Err(e) = data_source.send_reading(reading).await {
                    warn!("Error sending new reading to data-store: {e}")
                }
            }
            Event::DataServerMsg(msg) => radiators.update_if_relevant(msg),
        }
    }
}

struct Radiators {
    controller: Controller,
    small_bedroom: RadiatorState,
    large_bedroom: RadiatorState,
}

impl Radiators {
    fn update_if_relevant(&mut self, msg: M) {
        let Some(relevant_msg) = RelevantMsg::from(msg) else {
            return;
        };

        match relevant_msg {
            RelevantMsg::SmallBedroom(temp) => {
                if self.small_bedroom.should_update_given(temp) {
                    self.controller
                        .set_radiator_reference("small_bedroom:radiator", temp)
                }
            }
            RelevantMsg::LargeBedroom(temp) => {
                if self.large_bedroom.should_update_given(temp) {
                    self.controller
                        .set_radiator_reference("large_bedroom:radiator", temp)
                }
            }
        }
    }
}

enum Event {
    DataServerMsg(data_server::api::subscriber::SubMessage),
    ZigbeeReading(protocol::Reading),
}

impl RelevantMsg {
    fn from(msg: M) -> Option<Self> {
        use protocol::large_bedroom;
        use protocol::large_bedroom::desk as ldesk;
        use protocol::small_bedroom;
        use protocol::small_bedroom::desk as sdesk;
        use protocol::Reading;

        match msg {
            M::Reading(Reading::LargeBedroom(
                large_bedroom::Reading::Desk(ldesk::Reading::Temperature(temp)),
            )) => Some(RelevantMsg::LargeBedroom(temp as f64)),
            M::Reading(Reading::SmallBedroom(
                small_bedroom::Reading::Desk(sdesk::Reading::Temperature(temp)),
            )) => Some(RelevantMsg::SmallBedroom(temp as f64)),
            M::ErrorReport(_)
            | M::Reading(_)
            | M::AffectorControlled { .. } => None,
        }
    }
}
