#![no_std]

use postcard::experimental::schema::Schema;
use postcard_rpc::topic;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

topic!(ProtocolTopic, ProtocolMsg, "protocol");

// Unfortunately postcard::Schema is not a thing on heapless_vec
// so we have to try things differently
//
// also serde cannot handle >32 arrays so we gotta use a helper crate
#[serde_as]
#[derive(Schema, Serialize, Deserialize)]
pub struct StackVec {
    #[serde_as(as = "Bytes")]
    bytes: [u8; 250],
    len: u8,
}

impl StackVec {
    pub fn new() -> Self {
        StackVec {
            bytes: [0u8; 250],
            len: 0,
        }
    }
    pub fn capacity(&self) -> usize {
        self.bytes.len()
    }
    pub fn set_len(&mut self, len: usize) {
        self.len = len.min(self.capacity()) as u8;
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes[..self.len as usize]
    }
}

impl core::ops::Deref for StackVec {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl core::ops::DerefMut for StackVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

/// contains an encoded protocol::Msg
#[derive(Schema, Deserialize, Serialize)]
pub struct ProtocolMsg(pub StackVec);

impl<const M: usize> From<protocol::Msg<M>> for ProtocolMsg {
    fn from(value: protocol::Msg<M>) -> Self {
        let mut vec = StackVec::new();
        let len = value.encode_slice(&mut vec.bytes).len();
        vec.len = len as u8;
        ProtocolMsg(vec)
    }
}
