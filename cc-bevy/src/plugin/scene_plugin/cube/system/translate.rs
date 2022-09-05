use bevy::prelude::*;
use cc_core::{
    cube::{Constraint, Point},
    Diff,
};
use std::time::Duration;

use super::super::{super::view::GridView, component::Cubic};

#[derive(Component, Debug)]
pub struct TranslatePosition {
    elapse: Timer,
    parameters: PositionParameters,
}

impl TranslatePosition {
    pub fn make(cube: &Cubic, diff: &Diff, duration: Duration) -> Option<Self> {
        if let Some(position) = diff.position {
            return Some(TranslatePosition {
                elapse: Timer::new(duration, false),
                parameters: PositionParameters::Move(cube.position, position),
            });
        }

        let movement = diff.movement.unwrap_or(cube.movement);
        let constraint = diff.constraint.unwrap_or(cube.constraint);
        if constraint == Constraint::Stop || movement.is_none() {
            return Some(TranslatePosition {
                elapse: Timer::new(Duration::from_secs(0), false),
                parameters: PositionParameters::Stop,
            });
        }

        let movement = movement.unwrap();
        let limit = match constraint {
            Constraint::Slap => 0.5,
            Constraint::Lock => 0.05,
            _ => return None,
        };

        Some(TranslatePosition {
            elapse: Timer::new(duration, true),
            parameters: PositionParameters::Spin(cube.position, movement.into(), limit),
        })
    }
}

#[derive(Debug)]
enum PositionParameters {
    Move(Point, Point),      // (from, to)
    Spin(Point, Point, f32), // (from, delta, limit)
    Stop,
}

pub fn position_system(
    mut commands: Commands,
    mut cubes: Query<(Entity, &mut TranslatePosition, &mut Transform)>,
    view: Res<GridView>,
    time: Res<Time>,
) {
    let delta = time.delta();
    let mapper = view.mapping();
    for (id, mut translate, mut transform) in cubes.iter_mut() {
        use PositionParameters::*;
        if translate.elapse.tick(delta).finished() {
            match translate.parameters {
                Move(_, to) => {
                    transform.translation = mapper.absolute(&to).extend(0.);
                }
                Spin(from, _, _) => {
                    transform.translation = mapper.absolute(&from).extend(0.);
                }
                Stop => {
                    commands.entity(id).remove::<TranslatePosition>();
                }
            }
        } else {
            match translate.parameters {
                Move(from, to) => {
                    let percent = translate.elapse.percent();
                    let source = mapper.absolute(&from).extend(0.);
                    let target = mapper.absolute(&to).extend(0.);
                    transform.translation = source + (target - source) * percent;
                }
                Spin(from, delta, limit) => {
                    let percent = translate.elapse.percent();
                    let percent = (1.0 - percent).min(percent).min(limit);
                    let source = mapper.absolute(&from).extend(0.);
                    let delta = mapper.absolute(&delta).extend(0.);
                    transform.translation = source + delta * percent;
                }
                Stop => {
                    commands.entity(id).remove::<TranslatePosition>();
                }
            }
        }
    }
}