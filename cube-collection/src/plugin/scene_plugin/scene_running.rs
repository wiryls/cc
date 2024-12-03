use bevy::prelude::*;

use super::{
    common::{bundle, component, system},
    model,
    view::{GridView, ViewRect, ViewUpdated},
    SceneState,
};

pub fn setup(app: &mut App) {
    app.add_event::<WorldChanged>()
        .add_systems(OnEnter(SceneState::Running), setup_world)
        .add_systems(
            Update,
            (
                system::position,
                system::realpha,
                system::recolor,
                system::reshape,
            )
                .run_if(resource_exists::<model::World>)
                .before(system::state),
        )
        .add_systems(
            Update,
            system::state.run_if(resource_exists::<model::World>),
        )
        .add_systems(
            PostUpdate,
            (
                switch_world.run_if(on_event::<WorldChanged>),
                system::self_adaption.run_if(on_event::<ViewUpdated>),
            ),
        );
}

#[derive(Event)]
pub enum WorldChanged {
    Reset,
    Restart,
    Next,
    Last,
}

fn setup_world(mut change_world: EventWriter<WorldChanged>) {
    change_world.send(WorldChanged::Restart);
}

fn switch_world(
    mut commands: Commands,
    entities: Query<Entity, With<component::Earthbound>>,
    mut view: ResMut<GridView>,
    mut world_seeds: ResMut<model::Seeds>,
    mut world_changed: EventReader<WorldChanged>,
) {
    let got = !world_changed.is_empty();
    for event in world_changed.read() {
        use WorldChanged::*;
        match event {
            Reset => world_seeds.reset(),
            Restart => {}
            Next => drop(world_seeds.next()),
            Last => drop(world_seeds.last()),
        }
    }

    if let Some(seed) = got.then(|| world_seeds.current()).flatten() {
        // [0] remove all old objects
        entities
            .iter()
            .for_each(|i| commands.entity(i).despawn_recursive());

        // [1] update grid
        view.set_source(ViewRect {
            top: 0,
            bottom: seed.size.height,
            left: 0,
            right: seed.size.width,
        });
        let mapper = view.mapping();

        // [2] create new world
        let world = model::World::new(&seed);
        bundle::hello_world(&mut commands, &world, &mapper);
        commands.insert_resource(world);
    }
}
