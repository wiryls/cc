use bevy::prelude::*;
mod view;
use view::{GridLocator, LocatorUpdated};
mod debug;
use debug::DebugPlugin;

#[derive(Component, bevy_inspector_egui::Inspectable)]
struct Cube(i32, i32);

fn resize(
    mut query: Query<(&Cube, &mut Transform)>,
    mut relocator_updated: EventReader<LocatorUpdated>
) {
    for e in relocator_updated.iter().last() {
        let value = e.mapper.scale(0.9);
        let scale = Vec3::new(value, value, 0.);
        for (cube, mut transform) in query.iter_mut() {
            transform.scale =  scale;
            transform.translation = e.mapper.locate(cube.0, cube.1, 0);
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut locator: ResMut<GridLocator>,
) {
    locator.set_grid(Size::new(10, 10));

    let mapper = locator.mapping();
    let scale = mapper.scale(0.9);

    for x in 0..10 {
        for y in 0..10 {
            commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.7, 0.7, 0.7),
                    ..default()},
                transform: Transform {
                    scale: Vec3::new(scale, scale, 0.),
                    translation: mapper.locate(x, y, 0.),
                    ..default()},
                ..default()})
            .insert(Cube(x, y));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(view::ViewPlugin)
        .add_plugin(DebugPlugin)
        .add_startup_system(setup_scene)
        .add_system(resize)
        .run();
}