use nannou::noise::NoiseFn;
use nannou::prelude::*;
use nannou::rand::rand::Rng;

struct Hanger {
    start: Vector2,
    angle: f32,
    length: f32,
    ang_vel: f32,
}
impl Hanger {
    fn position(&self) -> Vector2 {
        self.start
            + Vector2 {
                x: self.angle.cos(),
                y: self.angle.sin(),
            } * self.length
    }
    fn update(&mut self, f: Vector2, g: Vector2, dt: f32) {
        let unit_r = Vector2 {
            x: self.angle.cos(),
            y: self.angle.sin(),
        };
        let unit_t = Vector2 {
            x: -unit_r.y,
            y: unit_r.x,
        };
        let ang_acc = (f + g).dot(unit_t);

        self.ang_vel += ang_acc * dt;
        self.ang_vel *= FRICTION;
        self.angle += self.ang_vel * dt;
    }
}

struct Model {
    hangers: Vec<Hanger>,
}
const N: usize = 100;
const WIND_VEL: f32 = 0.9;
const WIND_MAG: f32 = 7.0;
const GRAVITY: Vector2 = Vector2 { x: 0.0, y: -60.0 };
const FRICTION: f32 = 0.99;

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let mut rng = nannou::rand::rand::thread_rng();
    #[allow(deprecated)]
    let normal = nannou::rand::rand::distributions::Normal::new(0.0, 1.0);

    Model {
        hangers: (0..N)
            .map(|i| Hanger {
                start: Vector2 {
                    x: i as f32 * 10.0,
                    y: 0.0,
                },
                length: 200.0 + rng.sample(normal) as f32 * 50.0,
                angle: 3.0 * PI / 2.0, //+ rng.sample(normal) as f32 * (PI / 100.0),
                ang_vel: 0.0,          // + rng.sample(normal) as f32 * (PI / 100.0),
            })
            .collect(),
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(app, model, upd),
        _ => (),
    }
}

fn update(_app: &App, model: &mut Model, upd: Update) {
    let dt = upd.since_last.as_secs_f32();
    let elapsed = upd.since_start.as_secs_f32();

    let noise = nannou::noise::Perlin::new();
    let start = elapsed * WIND_VEL;

    for hanger in &mut model.hangers {
        let pos = hanger.position();
        let wind_x = noise.get([5.0 + start as f64 + (pos.x * WIND_VEL * 0.008) as f64, 0.0])
            as f32
            * WIND_MAG;
        hanger.update(
            Vector2 {
                x: wind_x as f32,
                y: 0.0,
            },
            GRAVITY,
            dt,
        );
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(rgb8(244, 234, 172));
    let win = app.window_rect();
    let draw = app.draw();
    let draw = draw.translate(Vector3::new(-win.x.len() / 2.0, 200.0, 0.0));

    for hanger in &model.hangers {
        draw.line()
            .start(hanger.start)
            .end(hanger.position())
            .weight(6.0 * (hanger.angle * 2.0).sin().abs() + 0.1)
            //.color(rgb8(56, 26, 6));
            .color(rgb8(238, 168, 0));
        draw.ellipse()
            .xy(hanger.position())
            .color(rgb8(197, 50, 0))
            .w_h(10.0, 10.0);
    }

    draw.to_frame(app, &frame).unwrap();
    frame.submit();
}
