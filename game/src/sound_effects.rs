use bevy::prelude::{App, AssetServer, Commands, Handle, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Startup};
use bevy_kira_audio::{Audio, AudioControl, AudioSource, DynamicAudioChannels};

pub struct SoundEffectsPlugin;

#[derive(Resource)]
pub struct AudioCache {
    pub coin_pickup: Handle<AudioSource>,
    pub game_loop: Handle<AudioSource>,
    // pub running: Handle<AudioSource>,
    pub jumping: Handle<AudioSource>,
    pub falling: Handle<AudioSource>,
}

impl Plugin for SoundEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio_system);
        app.add_systems(Startup, start_background_audio.after(setup_audio_system));
    }
}

fn setup_audio_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio_assets = AudioCache {
        coin_pickup: asset_server.load("sounds/coin-pickup.mp3"),
        game_loop: asset_server.load("sounds/game-loop-2.mp3"),
        // running: asset_server.load("sounds/running.mp3"),
        jumping: asset_server.load("sounds/jumping.mp3"),
        falling: asset_server.load("sounds/falling.mp3"),
    };

    commands.insert_resource(audio_assets);
}

fn start_background_audio(mut audio: ResMut<DynamicAudioChannels>, assets: Res<AudioCache>) {
    audio.create_channel("loop").play(assets.game_loop.clone()).looped();
}
