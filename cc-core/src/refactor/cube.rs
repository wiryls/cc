use std::{
    borrow::Borrow,
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use super::{
    extension::CollisionExtension,
    item::Item,
    kind::Kind,
    lookup::{BitmapCollision, Collision, HashSetCollision},
    motion::{Agreement, Motion},
    movement::{Constraint, Movement},
    neighborhood::{self, Adjacence, Neighborhood},
    point::Point,
};

/////////////////////////////////////////////////////////////////////////////
// export

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Collection {
    area: Rc<Area>,        // background and obstacles
    sets: Vec<Cube>,       // cubes (sets of units)
    link: Vec<Vec<usize>>, // groups to link
    view: Box<[Unit]>,     // a buffer of output
}

#[allow(dead_code)]
impl Collection {
    pub fn absorb(&mut self) -> &mut Self {
        let number_of_cubes = self.number_of_cubes();
        let unstable = (0..number_of_cubes)
            .filter_map(|index| Living::make(index, self).filter(Living::unstable));

        let faction = Faction::new::<Living, _, _>(self, unstable.clone());
        let mut connect = Connection::new(self);

        let mut visit = vec![false; number_of_cubes];
        let mut queue = VecDeque::with_capacity(number_of_cubes);

        // connect all adjacent cubes.
        for cube in unstable {
            if !visit[cube.index] {
                visit[cube.index] = true;
                queue.push_back(cube);
            }

            while let Some(cube) = queue.pop_back() {
                for other in faction.neighbors(&cube) {
                    if !visit[other.index] {
                        connect.join(&cube, &other);
                        visit[other.index] = true;
                        queue.push_back(other);
                    }
                }
            }
        }

        // try to absorb each others.
        for group in connect.groups() {
            let mut arena = Arena::new(self);
            for &index in group.iter() {
                if !arena.input(index) {
                    break;
                }
            }
            use ArenaResult::*;
            match arena.output() {
                Have(kind) => self.link_to_first(group, kind),
                Draw => self.mark_balanced(group),
                None => {}
            }
        }

        // do some cleaning.
        self.retain_alive();
        self
    }

    pub fn input(&mut self, input: Option<Movement>) {
        // clear and
        // update movement with input.
        self.link.clear();
        for cube in self.sets.iter_mut().filter(|cube| cube.kind == Kind::Green) {
            cube.movement = input;
        }

        // prepare
        let number_of_cubes = self.number_of_cubes();
        let unstable = (0..number_of_cubes).filter_map(|index| Living::make(index, self));
        let faction = Faction::new::<Living, _, _>(self, unstable);
        let mut connect = Connection::new(self);

        let moving = (0..number_of_cubes)
            .filter_map(|index| Moving::make(index, self))
            .collect::<Vec<_>>();

        for cube in moving.iter() {
            let mut blocked = cube.frontlines().any(|o| self.area.blocked(o));

            if !blocked {
                let neighbors = faction.neighbors_in_front(cube).collect::<HashSet<_>>();
                for other in neighbors.iter().map(Living::as_moving) {
                    blocked = !matches!(other, Some(other) if cube.movement() == other.movement() || cube.linkable(&other));
                    if blocked {
                        break;
                    }
                }

                if !blocked {
                    // for other in neighbors.iter() {
                    //     if cube.movement() != other.movement() && cube.linkable(&other) {
                    //         blocked = true;
                    //         connect.join(cube, &other);
                    //         // TODO: mark other as Stop.
                    //         // TODO: add other to seeds.
                    //     }
                    // }
                }

                if !blocked {
                    for other in neighbors {
                        if let Some(other) = other.as_moving() {
                            if cube.movement() == other.movement() {
                                // TODO: mark cube as a successor of other.
                            }
                        }
                    }
                }
            }

            if blocked {
                // TODO: mark cube as Stop.
                // TODO: add cube to seeds.
            }
        }

        todo!()
    }

    pub fn next(&mut self) {
        // clean status
        // move to next status
        todo!()
    }

    fn retain_alive(&mut self) {
        self.sets.retain(Cube::alive);
    }

    fn link_to_first(&mut self, from: Vec<usize>, kind: Kind) {
        let cube = &mut self.sets;

        let units = {
            let capacity = from.iter().map(|&i| cube[i].units.len()).sum::<usize>();
            let mut units = Vec::with_capacity(capacity);
            for &i in from.iter() {
                units.append(&mut cube[i].units);
            }
            units
        };
        let motion = {
            let mut others = Vec::with_capacity(from.len());
            for &i in from.iter() {
                if cube[i].kind == kind {
                    others.push(Motion::take(&mut cube[i].motion));
                }
            }
            Motion::from_iter(others.into_iter())
        };
        let outlines = Outlines::new(&units).into();
        let movement = {
            let mut agreement = Agreement::new();
            for &i in from.iter() {
                if cube[i].kind == kind {
                    agreement.submit(cube[i].movement);
                    if agreement.fail() {
                        break;
                    }
                }
            }
            agreement.result().unwrap_or_default()
        };
        let constraint = {
            let mut constraint = Constraint::Free;
            for &i in from.iter() {
                if cube[i].kind == kind {
                    constraint = constraint.max(cube[i].constraint);
                }
            }
            constraint
        };

        if let Some(&index) = from.first() {
            cube[index] = Cube {
                kind,
                units,
                motion,
                outlines,
                balanced: false,
                movement,
                constraint,
            };
        }
    }

    fn mark_balanced(&mut self, whom: Vec<usize>) {
        for index in whom {
            self.sets[index].balanced = true;
        }
    }

    fn number_of_cubes(&self) -> usize {
        self.sets.len()
    }
}

/////////////////////////////////////////////////////////////////////////////
// internal

#[derive(Clone, Debug)]
struct Cube {
    kind: Kind,
    units: Vec<Unit>,
    motion: Motion,
    outlines: Rc<Outlines>,     // calculated boundary points
    balanced: bool,             // state of being unabsorbable
    movement: Option<Movement>, // original movement direction
    constraint: Constraint,     // state of movement
}

impl Cube {
    fn alive(&self) -> bool {
        !self.units.is_empty()
    }

    fn unstable(&self) -> bool {
        !self.balanced && self.kind != Kind::White
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Unit {
    index: usize,
    position: Point,
    neighborhood: Neighborhood,
}

#[derive(Debug)]
pub struct Area {
    cubes: Box<[(Point, Neighborhood)]>,
    impassable: BitmapCollision,
}

#[allow(dead_code)]
impl Area {
    pub fn new<'a, I>(width: usize, height: usize, it: I) -> Self
    where
        I: Iterator<Item = &'a [Point]>,
    {
        let cubes = {
            let build = |os: &'a [Point]| {
                let c = HashSetCollision::new(os.iter());
                os.iter().map(move |&o| (o, c.neighborhood(o)))
            };
            it.flat_map(build).collect::<Box<_>>()
        };

        let impassable = {
            let mut it = BitmapCollision::new(width, height);
            cubes.iter().for_each(|x| it.put(x.0));
            it
        };

        Self { cubes, impassable }
    }

    pub fn blocked(&self, point: Point) -> bool {
        self.impassable.hit(point)
    }

    pub fn iter(&self, offset: usize) -> AreaIter {
        AreaIter {
            offset,
            iterator: self.cubes.iter().enumerate(),
        }
    }
}

pub struct AreaIter<'a> {
    offset: usize,
    iterator: std::iter::Enumerate<std::slice::Iter<'a, (Point, Neighborhood)>>,
}

impl Iterator for AreaIter<'_> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .next()
            .map(|(index, (point, neighborhood))| Item {
                id: (index + self.offset).into(),
                kind: Kind::White,
                position: point.clone(),
                movement: None,
                constraint: Constraint::Free,
                neighborhood: neighborhood.clone(),
            })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Outlines {
    count: [usize; 3],
    slice: Box<[Point]>,
}

#[allow(dead_code)]
impl Outlines {
    pub fn new(units: &[Unit]) -> Self {
        const LBTR: [Adjacence; 4] = [
            Adjacence::LEFT,
            Adjacence::BOTTOM,
            Adjacence::TOP,
            Adjacence::RIGHT,
        ];

        // count:
        // [  0     1       2    3   ]
        // 0..left..bottom..top..right
        let mut count: [usize; 4] = Default::default();
        for unit in units.iter() {
            for (i, a) in LBTR.into_iter().enumerate() {
                if !unit.neighborhood.has(a) {
                    count[i] += 1;
                }
            }
        }
        for i in 1..count.len() {
            count[i] += count[i - 1];
        }

        let mut slice = {
            let first = Self::anchor(units);
            let total = count[3];
            vec![first; total]
        };
        let mut index = {
            let mut index = count.clone();
            index.rotate_right(1);
            index[0] = 0;
            index
        };
        for unit in units.iter() {
            for (i, a) in LBTR.into_iter().enumerate() {
                if !unit.neighborhood.has(a) {
                    let point = &mut slice[index[i]];
                    *point -= unit.position;
                    *point -= a.into();
                    index[i] += 1;
                }
            }
        }

        Self {
            count: [count[0], count[1], count[2]],
            slice: slice.into(),
        }
    }

    pub fn one(&self, anchor: Point, m: Movement) -> impl Iterator<Item = Point> + Clone + '_ {
        use Movement::*;
        match m {
            Left /***/ => &self.slice[ /*      **/ ..self.count[0]],
            Down /***/ => &self.slice[self.count[0]..self.count[1]],
            Up /*  **/ => &self.slice[self.count[1]..self.count[2]],
            Right /**/ => &self.slice[self.count[2].. /*      **/ ],
        }
        .iter()
        .map(move |o| anchor - *o)
    }

    pub fn all(&self, anchor: Point) -> impl Iterator<Item = Point> + Clone + '_ {
        (&self.slice[..]).iter().map(move |o| anchor - *o)
    }

    fn anchor(units: &[Unit]) -> Point {
        units.first().map(|u| u.position).unwrap_or_default()
    }
}

/////////////////////////////////////////////////////////////////////////////
// lookups

struct Faction<'a>(super::lookup::Faction, &'a Collection);

impl<'a> Faction<'a> {
    fn new<C, B, I>(collection: &'a Collection, it: I) -> Self
    where
        I: Iterator<Item = B> + Clone,
        B: Borrow<C>,
        C: Cubic<'a>,
    {
        let mut faction = super::lookup::Faction::with_capacity(
            it.clone()
                .map(|x| x.borrow().value().units.len())
                .sum::<usize>(),
        );
        for cube in it {
            let cubic = cube.borrow();
            let index = cubic.index();
            let value = cubic.value();
            for unit in value.units.iter() {
                faction.put(unit.position, index);
            }
        }
        Self(faction, collection)
    }

    fn get(&self, point: Point) -> Option<Living<'a>> {
        self.0
            .get(point)
            .and_then(|index| Living::make(index, self.1))
    }

    fn neighbors(&self, cube: &impl Cubic<'a>) -> impl Iterator<Item = Living> + Clone + '_ {
        let cube = cube.value();
        let anchor = Outlines::anchor(&cube.units);
        cube.outlines.all(anchor).filter_map(|o| self.get(o))
    }

    fn neighbors_in_front(&self, cube: &Moving<'a>) -> impl Iterator<Item = Living> + Clone + '_ {
        cube.frontlines().filter_map(|o| self.get(o))
    }
}

struct Connection<'a>(super::lookup::DisjointSet, &'a Collection);

impl<'a> Connection<'a> {
    fn new(collection: &'a Collection) -> Self {
        let number = collection.number_of_cubes();
        Self(super::lookup::DisjointSet::new(number), collection)
    }

    fn join<'b, L: Cubic<'b>, R: Cubic<'b>>(&mut self, this: &'b L, that: &'b R) {
        self.0.join(this.index(), that.index());
    }

    fn groups(self) -> super::lookup::DisjointSetGroups {
        self.0.groups()
    }
}

struct Arena<'a>([bool; 3], &'a Collection);

enum ArenaResult {
    Have(Kind),
    Draw,
    None,
}

impl<'a> Arena<'a> {
    fn new(collection: &'a Collection) -> Self {
        Self([false; 3], collection)
    }

    fn input(&mut self, i: usize) -> bool {
        // try expression is experimental :-(
        // ```
        // try { Self::kind_to_index(self.1.sets.get(i)?.kind)? }
        // ```
        if let Some(index) = self.1.sets.get(i).and_then(|x| Self::kind_to_index(x.kind)) {
            self.0[index] = true;
        };

        !(self.0[0] && self.0[1] && self.0[2])
    }

    fn output(&self) -> ArenaResult {
        use ArenaResult::*;
        use Kind::*;
        match ((self.0[0] as usize) << 2)
            | ((self.0[1] as usize) << 1)
            | ((self.0[2] as usize) << 0)
        {
            0b111 => Draw,
            0b110 => Have(Blue),
            0b101 => Have(Red),
            0b011 => Have(Green),
            _else => None,
        }
    }

    const fn kind_to_index(kind: Kind) -> Option<usize> {
        use Kind::*;
        match kind {
            White => None,
            Red => Some(0),
            Blue => Some(1),
            Green => Some(2),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// Cubic

trait Cubic<'a> {
    fn index(&self) -> usize;
    fn value(&self) -> &'a Cube;
    fn owner(&self) -> &'a Collection;
}

impl<'a> Cubic<'a> for Living<'a> {
    fn index(&self) -> usize {
        self.index
    }

    fn value(&self) -> &'a Cube {
        self.value
    }

    fn owner(&self) -> &'a Collection {
        self.owner
    }
}

impl<'a> Cubic<'a> for Moving<'a> {
    fn index(&self) -> usize {
        self.index
    }

    fn value(&self) -> &'a Cube {
        self.value
    }

    fn owner(&self) -> &'a Collection {
        self.owner
    }
}

trait CubicExtension<'a>: Cubic<'a> {
    fn kind(&self) -> Kind;
    fn unstable(&self) -> bool;
    fn linkable<T: CubicExtension<'a>>(&self, that: &T) -> bool;
    fn absorbable<T: CubicExtension<'a>>(&self, that: &T) -> bool;
}

impl<'a, T: Cubic<'a>> CubicExtension<'a> for T {
    fn kind(&self) -> Kind {
        self.value().kind
    }

    fn unstable(&self) -> bool {
        self.value().unstable()
    }

    fn linkable<U: CubicExtension<'a>>(&self, that: &U) -> bool {
        self.kind().linkable(that.kind())
    }

    fn absorbable<U: CubicExtension<'a>>(&self, that: &U) -> bool {
        self.unstable() && that.unstable() && self.kind().absorbable(that.kind())
    }
}

#[derive(Debug)]
struct Living<'a> {
    index: usize,
    value: &'a Cube,
    owner: &'a Collection,
}

impl<'a> Living<'a> {
    fn make(index: usize, owner: &'a Collection) -> Option<Self> {
        owner
            .sets
            .get(index)
            .filter(|cube| cube.alive())
            .map(|value| Self {
                value,
                index,
                owner,
            })
    }

    fn as_moving(&self) -> Option<Moving<'a>> {
        self.value.movement.map(|movement| Moving {
            index: self.index,
            value: self.value,
            owner: self.owner,
            movement,
        })
    }
}

impl std::hash::Hash for Living<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl PartialEq for Living<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for Living<'_> {}

#[derive(Debug)]
struct Moving<'a> {
    index: usize,
    value: &'a Cube,
    owner: &'a Collection,
    movement: Movement,
}

impl<'a> Moving<'a> {
    fn make(index: usize, owner: &'a Collection) -> Option<Self> {
        owner
            .sets
            .get(index)
            .filter(|cube| cube.movement.is_some() && cube.alive())
            .map(|value| Self {
                value,
                index,
                owner,
                movement: value.movement.unwrap(),
            })
    }

    fn movement(&self) -> Movement {
        self.movement
    }

    fn frontlines(&self) -> impl Iterator<Item = Point> + Clone + 'a {
        let movement = self.movement;
        let anchor = Outlines::anchor(&self.value.units);
        self.value.outlines.one(anchor, movement)
    }
}

/////////////////////////////////////////////////////////////////////////////
// tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlines() {
        let outlines = Outlines::new(&[]);
        let actual = Vec::from_iter(outlines.all(Point::new(0, 0)));
        assert_eq!(actual, Vec::new());
        for movement in Movement::ALL {
            let actual = Vec::from_iter(outlines.one(Point::new(0, 0), movement));
            assert_eq!(actual, Vec::new());
        }

        let units = vec![
            Unit {
                index: 0,
                position: Point::new(0, 0),
                neighborhood: Neighborhood::from([Adjacence::BOTTOM].into_iter()),
            },
            Unit {
                index: 0,
                position: Point::new(0, 1),
                neighborhood: Neighborhood::from([Adjacence::TOP].into_iter()),
            },
        ];
        let outlines = Outlines::new(&units);

        let expected = vec![Point::new(0, 1), Point::new(0, 2)];
        let actual = Vec::from_iter(outlines.one(Point::new(1, 1), Movement::Left));
        assert_eq!(actual, expected);

        let expected = vec![Point::new(2, 1), Point::new(2, 2)];
        let actual = Vec::from_iter(outlines.one(Point::new(1, 1), Movement::Right));
        assert_eq!(actual, expected);

        let expected = vec![Point::new(1, 0)];
        let actual = Vec::from_iter(outlines.one(Point::new(1, 1), Movement::Up));
        assert_eq!(actual, expected);

        let expected = vec![Point::new(1, 3)];
        let actual = Vec::from_iter(outlines.one(Point::new(1, 1), Movement::Down));
        assert_eq!(actual, expected);
    }
}
