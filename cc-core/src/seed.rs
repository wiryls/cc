use super::common::Point;
use super::model::{Movement, Type};

pub struct Seed {
    pub info: Info,
    pub size: Size,
    pub cubes: Vec<Cube>,
    pub destnations: Vec<Point>,
}

pub struct Info {
    pub title: String,
    pub author: String,
}

pub struct Cube {
    pub kind: Type,
    pub body: Vec<Point>,
    pub command: Option<Command>,
}

pub struct Size {
    pub width: i32,
    pub height: i32,
}

pub struct Command {
    pub is_loop: bool,
    pub movements: Vec<(Movement, usize)>,
}