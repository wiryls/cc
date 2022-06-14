use super::Point;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Adjacence(u8);

impl Adjacence {
    pub const LEFT /*         **/: Adjacence = Adjacence(0b_10000000);
    pub const LEFT_TOP /*     **/: Adjacence = Adjacence(0b_01000000);
    pub const TOP /*          **/: Adjacence = Adjacence(0b_00100000);
    pub const RIGHT_TOP /*    **/: Adjacence = Adjacence(0b_00010000);
    pub const RIGHT /*        **/: Adjacence = Adjacence(0b_00001000);
    pub const RIGHT_BOTTOM /* **/: Adjacence = Adjacence(0b_00000100);
    pub const BOTTOM /*       **/: Adjacence = Adjacence(0b_00000010);
    pub const LEFT_BOTTOM /*  **/: Adjacence = Adjacence(0b_00000001);
}

pub trait Adjacent {
    fn near(&self, m: Adjacence) -> Self;
    fn step(&mut self, m: Adjacence) -> &mut Self;
}

impl Adjacent for Point {
    fn near(&self, m: Adjacence) -> Self {
        let mut next = self.clone();
        next.step(m);
        next
    }

    fn step(&mut self, m: Adjacence) -> &mut Self {
        match m {
            Adjacence::LEFT => self.x -= 1,
            Adjacence::LEFT_TOP => {
                self.x -= 1;
                self.y -= 1;
            }
            Adjacence::TOP => self.y -= 1,
            Adjacence::RIGHT_TOP => {
                self.x += 1;
                self.y -= 1;
            }
            Adjacence::RIGHT => self.x += 1,
            Adjacence::RIGHT_BOTTOM => {
                self.x += 1;
                self.y += 1;
            }
            Adjacence::BOTTOM => self.y += 1,
            Adjacence::LEFT_BOTTOM => {
                self.x -= 1;
                self.y += 1;
            }
            _ => (),
        }
        self
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Neighborhood(u8);

impl Neighborhood {
    pub const AROUND: [Adjacence; 8] = [
        Adjacence::LEFT,
        Adjacence::LEFT_TOP,
        Adjacence::TOP,
        Adjacence::RIGHT_TOP,
        Adjacence::RIGHT,
        Adjacence::RIGHT_BOTTOM,
        Adjacence::BOTTOM,
        Adjacence::LEFT_BOTTOM,
    ];

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(it: impl Iterator<Item = Adjacence>) -> Self {
        let mut n = Self::new();
        it.for_each(|a| n.set(a));
        n
    }

    pub fn set(&mut self, mask: Adjacence) {
        self.0 |= mask.0;
    }

    pub fn unset(&mut self, mask: Adjacence) {
        self.0 &= !mask.0;
    }

    pub fn has(&self, mask: Adjacence) -> bool {
        self.0 & mask.0 != 0
    }

    pub fn states(&self) -> [bool; 8] {
        Neighborhood::AROUND.map(|mask| (self.0 & mask.0) != 0)
    }
}