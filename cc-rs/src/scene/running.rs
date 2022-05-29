use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;

use super::state::State;
use crate::extra::grid::{GridPlugin, GridUpdated, GridView};
use crate::model::{bundle, cube, detail, seed};

/// - input: ```Res<seed::Seeds>```
/// - output: none
pub struct RunningScene;
impl Plugin for RunningScene {
    fn build(&self, app: &mut App) {
        app.add_plugin(GridPlugin)
            .add_event::<WorldChanged>()
            .add_enter_system(State::Running, setup_world)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(State::Running)
                    .with_system(switch_world.run_on_event::<WorldChanged>())
                    .with_system(regrid.run_on_event::<GridUpdated>())
                    .into(),
            );
    }
}

pub enum WorldChanged {
    Reset,
    Next,
}

fn setup_world(mut reset: EventWriter<WorldChanged>) {
    reset.send(WorldChanged::Reset)
}

fn switch_world(
    mut commands: Commands,
    entities: Query<Entity, With<cube::Live>>,
    mut view: ResMut<GridView>,
    mut world_seeds: ResMut<seed::Seeds>,
    mut world_changed: EventReader<WorldChanged>,
) {
    let got = !world_changed.is_empty();
    for event in world_changed.iter() {
        if let WorldChanged::Next = event {
            world_seeds.next();
        }
    }

    match got.then(|| world_seeds.current()).flatten() {
        None => return,
        Some(seed) => {
            // [0] update grid view
            let rect = Rect {
                left: 0,
                right: seed.size.width,
                top: 0,
                bottom: seed.size.height,
            };
            view.set_source(rect);

            // [1] remove old object
            entities.for_each(|i| commands.entity(i).despawn_recursive());

            // [2] create cubes
            let mapper = view.mapping();
            for c in &seed.cubes {
                let bulder = bundle::CubeBuilder::new(&c);
                bulder.build(&mut commands, mapper);
            }

            // for o in &seed.destnations {
            //     commands
            //         .spawn_bundle(SpriteBundle {
            //             sprite: Sprite {
            //                 color: Color::rgb(0.1, 0.1, 0.1),
            //                 ..default()
            //             },
            //             transform: Transform {
            //                 scale: Vec3::new(scale, scale, 1.),
            //                 translation: mapper.locate(o.x, o.y, 0.),
            //                 ..default()
            //             },
            //             ..default()
            //         })
            //         .insert(cube::GridPoint { x: o.x, y: o.y })
            //         .insert(cube::Live);
            // }
        }
    }
}

fn regrid(
    mut query: Query<(&cube::GridPoint, &cube::Pattern, &mut Transform, &mut Path)>,
    mut grid_updated: EventReader<GridUpdated>,
) {
    for e in grid_updated.iter().last() {
        let value = e.mapper.unit();
        for (cube, style, mut transform, mut shape) in query.iter_mut() {
            transform.translation = e.mapper.locate(cube.x, cube.y, 0);
            *shape = ShapePath::build_as(&shapes::Polygon {
                points: detail::make_boundaries(value, 0.98, detail::Near::from(style)),
                closed: true,
            });
        }
    }
}