use bevy::prelude::*;
use bevy_prototype_lyon::plugin;

/// A wrapper of ShapePlugin.
pub struct ShapePlugin;
impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(plugin::ShapePlugin);
    }
}
