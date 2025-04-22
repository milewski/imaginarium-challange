use std::time::{Duration};

use bevy::app::{App, Plugin};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::utils::info;
// use bevy_simplenet::{ClientConfig, ClientEventFrom, ClientReport};
use serde::{Deserialize, Serialize};
// use wasm_timer::{SystemTime, UNIX_EPOCH};

// use shared::{DemoChannel, ServerChannel, ServerMessages};

// type DemoClient = bevy_simplenet::Client<DemoChannel>;
// type DemoClientEvent = bevy_simplenet::ClientEventFrom<DemoChannel>;

pub const PROTOCOL_ID: u64 = 7;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

pub struct NetworkPlugin;

fn client_factory() -> bevy_simplenet::ClientFactory<DemoChannel> {
    bevy_simplenet::ClientFactory::<DemoChannel>::new("demo")
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let client = client_factory().new_client(
            enfync::builtin::native::TokioHandle::default(),
            url::Url::parse("ws://127.0.0.1:48888/ws").unwrap(),
            bevy_simplenet::AuthRequest::None {
                client_id: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
            },
            ClientConfig::default(),
            ()
        );
        //
        // app.insert_resource(client);
        // app.add_systems(Update, client_sync_players);
    }
}

// fn client_sync_players(
//     mut client: ResMut<DemoClient>,
// ) {
//     while let Some(client_event) = client.next() {
//         match client_event {
//             DemoClientEvent::Report(report) => {
//                 match report {
//                     ClientReport::Connected => {
//                         println!("Connected !")
//                     }
//                     ClientReport::Disconnected => {}
//                     ClientReport::ClosedByServer(_) => {}
//                     ClientReport::ClosedBySelf => {}
//                     ClientReport::IsDead(_) => {}
//                 }
//             },
//             DemoClientEvent::Msg(_) => {},
//             DemoClientEvent::Response(_, _) => {},
//             DemoClientEvent::Ack(_) => {},
//             DemoClientEvent::Reject(_) => {},
//             DemoClientEvent::SendFailed(_) => {},
//             DemoClientEvent::ResponseLost(_) => {},
//         }
//     }
// }
