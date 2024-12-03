use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use cube_core::cube::Point;

pub fn setup(app: &mut App) {
    app.init_resource::<GridView>()
        .add_event::<ViewUpdated>()
        .add_systems(Startup, setup_camera)
        .add_systems(PreUpdate, update_gridview.run_if(on_event::<WindowResized>));
}

#[derive(Event)]
pub struct ViewUpdated {
    pub mapper: ViewMapper,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ViewRect<T> {
    pub top: T,
    pub bottom: T,
    pub left: T,
    pub right: T,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn update_gridview(
    windows: Query<Entity, With<PrimaryWindow>>,
    mut view: ResMut<GridView>,
    mut window_resized: EventReader<WindowResized>,
    mut mapper_updated: EventWriter<ViewUpdated>,
) {
    const PADDING_RATE: f32 = 0.04;

    // multiple windows are not supported, we just watch the primary one.
    if let Ok(window) = windows.get_single() {
        while let Some(event) = window_resized
            .read()
            .filter(|x| x.window.index() == window.index())
            .last()
        {
            // rect is defined by camera's ScalingMode::WindowSize
            let p = event.width.min(event.height) * PADDING_RATE;
            let w = event.width * 0.5 - p;
            let h = event.height * 0.5 - p;
            let r = ViewRect {
                top: h,
                bottom: -h,
                left: -w,
                right: w,
            };

            if view.set_target(r) && view.available() {
                let mapper = view.mapping().clone();
                mapper_updated.send(ViewUpdated { mapper });
            }
        }
    }
}

#[derive(Default, Resource)]
pub struct GridView {
    // input
    source: Option<ViewRect<i32>>,
    target: Option<ViewRect<f32>>,
    // output
    mapper: ViewMapper,
}

impl GridView {
    pub fn set_source(&mut self, source: ViewRect<i32>) -> bool {
        let update = match self.source {
            None => true,
            Some(rect) => rect != source,
        };
        if update {
            self.source = Some(source);
            self.remap();
        }
        update
    }

    pub fn set_target(&mut self, target: ViewRect<f32>) -> bool {
        let update = match self.target {
            None => true,
            Some(rect) => rect != target,
        };
        if update {
            self.target = Some(target);
            self.remap();
        }
        update
    }

    pub fn available(&self) -> bool {
        self.source.is_some() && self.target.is_some()
    }

    pub fn mapping(&self) -> &ViewMapper {
        &self.mapper
    }

    fn remap(&mut self) {
        if let (Some(source), Some(target)) = (self.source, self.target) {
            let tw = target.right - target.left;
            let th = target.top - target.bottom;
            let sw = (source.right - source.left) as f32;
            let sh = (source.bottom - source.top) as f32;
            let unit = f32::min(tw / sw, th / sh);

            self.mapper = ViewMapper {
                source: (source.left as i32, source.top as i32),
                target: Vec2 {
                    x: target.left + (tw - sw * unit) / 2.,
                    y: target.top - (th - sh * unit) / 2.,
                },
                unit,
            };
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ViewMapper {
    source: (i32, i32),
    target: Vec2,
    unit: f32,
}

impl ViewMapper {
    pub const fn unit(&self) -> f32 {
        self.unit
    }

    #[allow(dead_code)]
    pub fn flip(&self, o: &impl Mappable) -> Vec2 {
        o.delta(self.source).into()
    }

    pub fn scale(&self, o: &impl Mappable) -> Vec2 {
        o.scale(self.unit).into()
    }

    pub fn locate(&self, o: &impl Mappable) -> Vec2 {
        let mut output = Vec2::from(o.delta(self.source));
        output *= self.unit;
        output += self.target;
        output
    }
}

pub trait Mappable {
    fn scale(&self, factor: f32) -> (f32, f32);
    fn delta(&self, source: (i32, i32)) -> (f32, f32);
}

impl Mappable for (i32, i32) {
    fn scale(&self, factor: f32) -> (f32, f32) {
        (self.0 as f32 * factor, self.1 as f32 * -factor)
    }

    fn delta(&self, source: (i32, i32)) -> (f32, f32) {
        ((self.0 - source.0) as f32, (source.1 - self.1) as f32)
    }
}

impl Mappable for (f32, f32) {
    fn scale(&self, factor: f32) -> (f32, f32) {
        (self.0 * factor, self.1 * -factor)
    }

    fn delta(&self, source: (i32, i32)) -> (f32, f32) {
        (self.0 - source.0 as f32, source.1 as f32 - self.1)
    }
}

impl Mappable for (usize, usize) {
    fn scale(&self, factor: f32) -> (f32, f32) {
        (self.0 as f32 * factor, self.1 as f32 * -factor)
    }

    fn delta(&self, source: (i32, i32)) -> (f32, f32) {
        (
            (self.0 as i64 - source.0 as i64) as f32,
            (source.1 as i64 - self.1 as i64) as f32,
        )
    }
}

impl Mappable for Point {
    fn scale(&self, factor: f32) -> (f32, f32) {
        (self.x as f32 * factor, self.y as f32 * -factor)
    }

    fn delta(&self, source: (i32, i32)) -> (f32, f32) {
        ((self.x - source.0) as f32, (source.1 - self.y) as f32)
    }
}

impl Mappable for Vec2 {
    fn scale(&self, factor: f32) -> (f32, f32) {
        (self.x as f32 * factor, self.y as f32 * -factor)
    }

    fn delta(&self, source: (i32, i32)) -> (f32, f32) {
        (self.x - source.0 as f32, source.1 as f32 - self.y)
    }
}
