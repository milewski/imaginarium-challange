use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DemoChannel;

// impl bevy_simplenet::ChannelPack for DemoChannel {
//     type ConnectMsg = ();
//     type ServerMsg = DemoServerMsg;
//     type ServerResponse = ();
//     type ClientMsg = ();
//     type ClientRequest = DemoClientRequest;
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DemoClientRequest {
    Select,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DemoServerMsg {
    Current(Option<u128>),
}

#[derive(Debug, Default, bincode::Encode, bincode::Decode)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum SystemMessages {
    PlayerPosition { coordinate: Coordinate },
}
