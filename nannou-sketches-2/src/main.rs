use std::ops::{Add, Mul};

use nannou::{geom::Range, prelude::*};

trait RiemannianDot {
    /// The dimension of the space.
    const DIM: usize;

    /// A quadratic form that should be positive definite.
    fn dot(self, other: Self) -> f32;
}

impl RiemannianDot for f32 {
    const DIM: usize = 1;

    fn dot(self, other: Self) -> f32 {
        self * other
    }
}

impl RiemannianDot for Vec2 {
    const DIM: usize = 2;

    fn dot(self, other: Self) -> f32 {
        self.dot(other)
    }
}

trait VectorSpace: Add<Self> + Mul<f32, Output = Self> + Copy + RiemannianDot {}
impl<T: Add<Self> + Mul<f32, Output = Self> + Copy + RiemannianDot> VectorSpace for T {}

#[derive(Clone, Copy)]
struct Pure2Tensor<V> {
    scalar_1: f32,
    generator_1: V,
    generator_2: V,
}

impl<V: VectorSpace> Pure2Tensor<V> {
    pub fn new(v1: V, v2: V) -> Self {
        Self {
            generator_1: v1,
            generator_2: v2,
            scalar_1: 1.0f32,
        }
    }

    fn scalar_1(&self) -> f32 {
        self.scalar_1
    }

    fn scalar_2(&self) -> f32 {
        // You have more degrees of freedom for higher-order tensors.
        1.0 / self.scalar_1
    }

    pub fn v1(&self) -> V {
        self.generator_1 * self.scalar_1()
    }

    pub fn v2(&self) -> V {
        self.generator_2 * self.scalar_2()
    }

    pub fn scale_v1(&mut self, by: f32) {
        self.scalar_1 *= by;
    }

    pub fn scale_v2(&mut self, by: f32) {
        self.scalar_1 /= by;
    }
}

struct Model {
    x_hat: Vec2,                                    // Screen space.
    dragging_background_from: Option<(Vec2, Vec2)>, // (starting x_hat, starting click). Screen space.
    mouse_position: Vec2,                           // Screen space.
    tensor: Pure2Tensor<f32>,                       // In coord system.
}

impl Model {
    fn x_hat(&self) -> Vec2 {
        self.x_hat
    }

    fn y_hat(&self) -> Vec2 {
        let [x, y] = self.x_hat.to_array();
        Vec2::new(-y, x)
    }
}

fn model(_app: &App) -> Model {
    Model {
        x_hat: Vec2::new(4.0, 0.0),
        dragging_background_from: None,
        tensor: Pure2Tensor::new(10.0, 10.0),
        mouse_position: Vec2::ZERO,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app
        .draw()
        .rotate(model.x_hat().angle())
        .scale(model.x_hat().length());

    draw.arrow()
        .start(Vec2::ZERO)
        .end(Vec2::X * 12.0)
        .color(BEIGE);
    draw.arrow()
        .start(Vec2::ZERO)
        .end(Vec2::Y * 12.0)
        .color(BROWN);
    draw.background().color(TURQUOISE);

    draw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, update_: Update) {
    // todo: model the tensor as a rectangle stapled to the coordinate axes, having a mass in the middle with strings attaching to the side of the rectangle.
    // area is fixed, sides are springs.
    // gravity changes as the user rotates the tensor.
    // the sides are springs and the area is constant.
}

fn window_event(app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        MouseMoved(mouse) => {
            model.mouse_position = mouse;
            if let Some((start_x_hat, start_mouse_position)) = model.dragging_background_from {
                let rot = start_mouse_position.angle_between(model.mouse_position);
                let scale = model.mouse_position.length() / start_mouse_position.length();
                model.x_hat = start_x_hat.rotate(rot) * scale;
            }
        }
        MousePressed(mouse_button) => {
            model.dragging_background_from = Some((model.x_hat, model.mouse_position));
        }
        MouseReleased(mouse_button) => {
            model.dragging_background_from = None;
        }
        _ => (),
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(update_) => update(app, model, update_),
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            window_event(app, model, event);
        }
        _ => (),
    }
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}
