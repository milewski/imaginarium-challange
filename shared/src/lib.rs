use std::hash::{Hash, Hasher};
use bevy::math::Vec3;
use bevy::prelude::Component;
use bincode::config::standard;
use bincode::error::DecodeError;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use tokio_tungstenite_wasm::Message;
#[cfg(not(target_arch = "wasm32"))]
use tungstenite::Message;

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, 0.0, self.y as f32)
    }

    pub fn drift_by(mut self, amount: i32) -> Coordinate {
        self.x -= amount;
        self.y -= amount;

        self
    }
}

impl From<Vec3> for Coordinate {
    fn from(value: Vec3) -> Self {
        Coordinate {
            x: value.x as i32,
            y: value.z as i32,
        }
    }
}

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct PlayerId(u32);

impl PlayerId {
    pub fn random() -> Self {
        PlayerId(fastrand::u32(..))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct PlayerData {
    pub id: PlayerId,
    pub balance: u32,
    pub position: Coordinate,
}

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Monument {
    pub id: u32,
    pub asset: String,
    pub description: String,
    pub position: Coordinate,
    pub under_construction: bool,
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub enum SystemMessages {
    Ping,
    Pong,
    Connected {
        id: PlayerId,
    },
    Welcome {
        data: PlayerData,
    },

    PlayerPosition { coordinate: Coordinate },
    EnemyPosition { id: PlayerId, coordinate: Coordinate },
    EnemyDisconnected { id: PlayerId },

    BuildMonumentRequest { prompt: String },
    MonumentCompleted { id: u32, asset: String },
    BuildMonument { monument: Monument },
    MainPlayerPickedUpToken,
    MainPlayerCurrentBalance { balance: u32 },
    MainPlayerSpawn { data: PlayerData },
    EnemyPlayerSpawn { data: PlayerData },
}

impl TryFrom<Message> for SystemMessages {
    type Error = DecodeError;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let (decoded, _) = bincode::decode_from_slice(message.into_data().as_ref(), standard())?;
        Ok(decoded)
    }
}

impl Into<Message> for SystemMessages {
    fn into(self) -> Message {
        Message::Binary(
            bincode::encode_to_vec(self, standard()).unwrap().into()
        )
    }
}
