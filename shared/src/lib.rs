use std::time::Duration;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Coordinate {
    x: u32,
    y: u32
}


