use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use clap::Parser;
use data_server::api::subscriber::ReconnectingClient;
use data_server::api::subscriber::SubMessage as M;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Opt {
    /// IP address where to subscribe for updates
    #[clap(long)]
    data_server: SocketAddr,

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
    logger::tracing::setup_unlimited();

    let mut data_server = ReconnectingClient::new(
        args.data_server,
        env!("CARGO_PKG_NAME").to_owned(),
    )
    .subscribe();

    let zigbee =
        zigbee_bridge::Controller::start_bridge(args.mqtt_ip, "temp-bridge");
    let mut lb_ratiator = RadiatorState::default();
    let mut sb_ratiator = RadiatorState::default();

    loop {
        let msg = data_server.next().await;
        let Some(relevant_msg) = RelevantMsg::from(msg) else {
            continue;
        };

        match relevant_msg {
            RelevantMsg::SmallBedroom(temp) => {
                if sb_ratiator.should_update_given(temp) {
                    zigbee
                        .set_radiator_reference("small_bedroom:radiator", temp)
                }
            }
            RelevantMsg::LargeBedroom(temp) => {
                if lb_ratiator.should_update_given(temp) {
                    tracing::info!("setting temp");
                    zigbee
                        .set_radiator_reference("large_bedroom:radiator", temp)
                }
            }
        }
    }
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
