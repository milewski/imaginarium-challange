use bevy::math::Vec3;
use bevy::prelude::{Component, Resource};

#[derive(Debug, Default, Copy, Clone, bincode::Encode, bincode::Decode)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, 0.0, self.y as f32)
    }
}

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct PlayerId(u32);

impl PlayerId {
    pub fn random() -> Self {
        PlayerId(fastrand::u32(..))
    }
}

#[derive(Resource, Debug, Copy, Clone, bincode::Encode, bincode::Decode)]
pub struct PlayerData {
    pub id: PlayerId,
    pub position: Coordinate,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum SystemMessages {
    Connected {
        id: PlayerId,
    },
    Welcome {
        data: PlayerData,
    },
    PlayerPosition {
        id: PlayerId,
        coordinate: Coordinate,
    },
    PlayerSpawn {
        data: PlayerData,
    },
}
