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

pub use affector::Registar as AffectorRegistar;
pub use data_source::handle_nodes;
pub use subscribe::spread_updates;

use crate::api;
use crate::api::SubMessage;

pub type Conn = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    api::Request,
    api::Response,
    Bincode<api::Request, api::Response>,
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
