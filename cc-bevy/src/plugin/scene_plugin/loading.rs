use bevy::prelude::*;
use iyes_loopless::prelude::*;

use super::SceneState;
use crate::model::seed::Seeds;
use crate::plugin::loader_plugin::{LevelLoadingUpdated, LoadLevels, LoaderPlugin};

pub fn setup_scene(app: &mut App) {
    app.add_plugin(LoaderPlugin)
        .add_enter_system(SceneState::Loading, loading_enter)
        .add_system(loading_updated.run_in_state(SceneState::Loading));
}

fn loading_enter(mut commands: Commands) {
    commands.insert_resource(LoadLevels::new(r"level/index.toml"));
}

fn loading_updated(mut commands: Commands, mut status: EventReader<LevelLoadingUpdated>) {
    for event in status.iter() {
        use LevelLoadingUpdated::*;
        match event {
            Loading { total, done } => println!("Loading: {}/{}", done, total),
            Failure { which } => println!("Loading: {}", which),
            Success { seeds } => {
                commands.insert_resource(Seeds::from(seeds.clone()));
                commands.insert_resource(NextState(SceneState::Running));
            }
        }
    }
}
