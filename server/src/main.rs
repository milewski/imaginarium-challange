use std::collections::HashMap;
use std::error::Error;

use bevy_simplenet::ServerReport;

use shared::{Coordinate, DemoChannel};

#[derive(Default)]
struct PlayerData {
    position: Coordinate
}

#[derive(Default)]
struct Cache {
    clients: HashMap<u32, PlayerData>
}

fn server_factory() -> bevy_simplenet::ServerFactory<DemoChannel> {
    bevy_simplenet::ServerFactory::<DemoChannel>::new("demo")
}

type DemoServerEvent = bevy_simplenet::ServerEventFrom<DemoChannel>;

fn main() -> Result<(), Box<dyn Error>> {

    // simplenet server
    // - we use a baked-in address so you can close and reopen the server to test clients being disconnected
    let mut server = server_factory().new_server(
        enfync::builtin::native::TokioHandle::default(),
        "127.0.0.1:48888",
        bevy_simplenet::AcceptorConfig::Default,
        bevy_simplenet::Authenticator::None,
        bevy_simplenet::ServerConfig {
            heartbeat_interval: std::time::Duration::from_secs(6),  //slower than client to avoid redundant pings
            ..Default::default()
        },
    );

    loop {
        while let Some((client_id, event)) = server.next() {
            match event {
                DemoServerEvent::Report(report) => {
                    match report {
                        ServerReport::Connected(_, _) => {
                            println!("client connected {:?}", client_id)
                        }
                        ServerReport::Disconnected => {
                            println!("client disconnected {:?}", client_id)
                        }
                    }
                },
                DemoServerEvent::Msg(_) => {},
                DemoServerEvent::Request(_, _) => {},
            }
        }
    }

    Ok(())
}