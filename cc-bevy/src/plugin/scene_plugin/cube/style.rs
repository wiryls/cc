use bevy::prelude::*;
use cc_core::cube::{Kind, Neighborhood};

pub fn cube_color(kind: Kind) -> Color {
    match kind {
        Kind::White /* **/ => Color::rgb(1.0, 1.0, 1.0),
        Kind::Red /*   **/ => Color::rgb(0.9, 0.1, 0.1),
        Kind::Blue /*  **/ => Color::rgb(0.1, 0.1, 0.9),
        Kind::Green /* **/ => Color::rgb(0.1, 0.9, 0.1),
    }
}

pub fn cube_boundaries(pattern: Neighborhood, scale: f32, ratio: f32) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(12);

    let is_occupied = pattern.states();
    let max = scale * 0.5;
    let min = max * ratio.clamp(0., 1.);

    //    3      2                       0      3
    //     ┌────┬─────────────────────────┬────┐
    //     │    │                         │    │
    //     │    │                         │    │
    //     ├────┼─────────────────────────┼────┤
    //    0│    │1                       1│    │2
    //     │    │                         │    │
    //     │    │                         │    │
    //     │    │                         │    │
    //     │    │                         │    │
    //     │    │                         │    │
    //     │    │          (0, 0)         │    │
    //     │    │                         │    │
    //     │    │                         │    │
    //     │    │                         │    │
    //     │    │                         │    │
    //    2│    │1                       1│    │0
    //     ├────┼─────────────────────────┼────┤
    //     │    │                         │    │
    //     │    │                         │    │
    //     └────┴─────────────────────────┴────┘
    //    3      0                       2      3
    let mut v = [
        Vec2::new(-min, -max), // 0
        Vec2::new(-min, -min), // 1
        Vec2::new(-max, -min), // 2
        Vec2::new(-max, -max), // 3
    ];

    for i in 0..4 {
        for j in 0..4 {
            (v[j].x, v[j].y) = (v[j].y, -v[j].x);
        }

        match (
            is_occupied[(2 * i + 0)],
            is_occupied[(2 * i + 1)],
            is_occupied[(2 * i + 2) % is_occupied.len()],
        ) {
            (true, true, true) => {
                points.push(v[3]);
            }
            (true, _, true) => {
                points.push(v[0]);
                points.push(v[1]);
                points.push(v[2]);
            }
            (true, _, _) => {
                points.push(v[0]);
            }
            (_, _, true) => {
                points.push(v[2]);
            }
            _ => {
                points.push(v[1]);
            }
        }
    }

    points
}