use color_eyre::Result;
use protocol::Reading;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

mod affector;
pub mod client;
mod data_source;
mod subscribe;
mod watch;

pub use affector::Registar as AffectorRegistar;
pub use data_source::handle_nodes;
pub use subscribe::spread_updates;
pub use watch::node_watchdog;

use crate::api::subscriber::{self, SubMessage};

pub type Conn = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    subscriber::Request,
    subscriber::Response,
    Bincode<subscriber::Request, subscriber::Response>,
>;

#[derive(Debug)]
pub enum Event {
    NewSub {
        tx: mpsc::Sender<SubMessage>,
    },
    NewReading(Result<Reading, Box<protocol::Error>>),
    AffectorControlled {
        affector: protocol::Affector,
        controlled_by: String,
    },
}
