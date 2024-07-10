use bevy::color::palettes::css;
use bevy::prelude::*;
use cube_core::cube::{Adjacence, Kind, Movement, Neighborhood, Point};

pub const fn background_color() -> Color {
    Color::Srgba(css::WHITE)
}

pub const fn floor_color() -> Color {
    Color::srgb(0.2, 0.2, 0.2)
}

pub const fn destnation_color() -> Color {
    Color::Srgba(css::GRAY)
}

pub const fn cube_color(kind: Kind) -> Color {
    match kind {
        Kind::White /* **/ => Color::srgb(1.000, 1.000, 1.000),
        Kind::Red /*   **/ => Color::srgb(0.988, 0.512, 0.512),
        Kind::Blue /*  **/ => Color::srgb(0.582, 0.727, 0.945),
        Kind::Green /* **/ => Color::srgb(0.533, 0.859, 0.425),
    }
}

pub fn cube_boundaries(pattern: Neighborhood, scale: f32) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(12);

    let is_occupied = pattern.states();
    let max = 0.5;
    let min = max * scale.clamp(0., 1.);

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
            is_occupied[2 * i + 0],
            is_occupied[2 * i + 1],
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

pub struct BoundaryBuilder {
    width: usize,
    height: usize,
    marks: [Vec<bool>; 4],
}

impl BoundaryBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            marks: [
                vec![false; width],  // top
                vec![false; height], // right
                vec![false; width],  // bottom
                vec![false; height], // left
            ],
        }
    }

    pub fn put(&mut self, point: Point, neighbor: Neighborhood) {
        let inside = (0 <= point.x && point.x < self.width as i32)
            && (0 <= point.y && point.y < self.height as i32);
        let (x, y) = match inside {
            true => (point.x as usize, point.y as usize),
            false => return,
        };

        if y == 0 && neighbor.has(Adjacence::TOP) {
            self.marks[0][x] = true;
        }
        if x + 1 == self.width && neighbor.has(Adjacence::RIGHT) {
            self.marks[1][y] = true;
        }
        if y + 1 == self.height && neighbor.has(Adjacence::BOTTOM) {
            self.marks[2][x] = true;
        }
        if x == 0 && neighbor.has(Adjacence::LEFT) {
            self.marks[3][y] = true;
        }
    }

    pub fn build(&self, gap: f32) -> Vec<Vec2> {
        let mut out = Vec::new();
        let mut waver = SquareWaveGenerator::new(&mut out, 1.0, gap * 0.5);

        let w = self.width as f32;
        let h = self.height as f32;

        // fill top
        waver.turn(Vec2::new(0., 0.), Movement::Right);
        self.marks[0].iter().for_each(|&near| waver.put(near));

        // fill right
        waver.turn(Vec2::new(w, 0.), Movement::Down);
        self.marks[1].iter().for_each(|&near| waver.put(near));

        // fill bottom
        waver.turn(Vec2::new(w, h), Movement::Left);
        self.marks[2].iter().rev().for_each(|&near| waver.put(near));

        // fill left
        waver.turn(Vec2::new(0., h), Movement::Up);
        self.marks[3].iter().rev().for_each(|&near| waver.put(near));

        // transform from matrix indexes to cartesian coordinate system
        out.iter_mut().for_each(|v| v.y = -v.y);
        // keep clockwise orders
        out.reverse();

        out
    }
}

struct SquareWaveGenerator<'a> {
    step: f32,
    amplitude: f32,
    direction: Vec2,
    back: Vec2,
    left: Vec2,
    anchor: Vec2,
    last: bool,
    counter: usize,
    output: &'a mut Vec<Vec2>,
}

impl<'a> SquareWaveGenerator<'a> {
    fn new(output: &'a mut Vec<Vec2>, step: f32, amplitude: f32) -> Self {
        Self {
            step,
            amplitude,
            direction: Vec2::default(),
            back: Vec2::default(),
            left: Vec2::default(),
            anchor: Vec2::default(),
            last: false,
            counter: 0,
            output,
        }
    }

    fn put(&mut self, near: bool) {
        if self.counter == 0 {
            if near {
                self.output.push(self.anchor);
            } else {
                self.output.push(self.anchor + self.back + self.left);
            }
        } else if self.last != near {
            let step = self.anchor + self.direction * (self.counter as f32 * self.step);
            if near {
                // from far to near
                self.output.push(step - self.back + self.left);
                self.output.push(step - self.back);
            } else {
                // from near to far
                self.output.push(step + self.back);
                self.output.push(step + self.back + self.left);
            }
        }

        self.last = near;
        self.counter += 1;
    }

    fn turn(&mut self, anchor: Vec2, direction: Movement) {
        use Movement::*;
        self.direction = Vec2::from(match direction {
            Left /*  **/ => (-1., 0.),
            Down /*  **/ => (0., 1.),
            Up /*    **/ => (0., -1.),
            Right /* **/ => (1., 0.),
        });

        self.back = self.direction * -self.amplitude;
        self.left = Vec2::new(-self.back.y, self.back.x);
        self.anchor = anchor;
        self.counter = 0;
    }
}
