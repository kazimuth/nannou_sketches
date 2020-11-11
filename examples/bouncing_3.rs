use nannou::geom::Range;
use nannou::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

#[derive(Debug)]
struct Triangle {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    r: f32,
    angle: f32,
    ang_vel: f32,
}

struct Model {
    triangles: Vec<Triangle>,
    image: nannou::image::RgbImage,
}

const N: usize = 50;
const GRAVITY: Vector2<f32> = Vector2 { x: 0.0, y: -5.0 };

// domain is (-.5, .5) x (-.5, .5)
const SIM_BOUNDS: Rect<f32> = Rect {
    x: Range {
        start: -0.5,
        end: 0.5,
    },
    y: Range {
        start: -0.5,
        end: 0.5,
    },
};

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let mut rng: XorShiftRng = SeedableRng::seed_from_u64(12345);
    let triangles = (0..N)
        .map(|_| Triangle {
            pos: rng.gen::<Vector2<f32>>() - vec2(0.5f32, 0.5f32),
            vel: rng.gen::<Vector2<f32>>() - vec2(0.5f32, 0.5f32),
            r: rng.gen::<f32>() / 100.0 + 0.001,
            angle: rng.gen::<f32>() * 2.0 * PI,
            ang_vel: (rng.gen::<f32>() - 0.5) * 4.0 * PI,
        })
        .collect::<Vec<_>>();

    let image = nannou::image::open("bluebird.jpg").unwrap().to_rgb();

    Model { triangles, image }
}

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(model, upd),
        _ => (),
    }
}

fn update(model: &mut Model, upd: Update) {
    let dt = upd.since_last.as_secs_f32();
    for tri in model.triangles.iter_mut() {
        tri.vel += GRAVITY * dt;
    }
    for tri in model.triangles.iter_mut() {
        tri.pos += tri.vel * dt;
        tri.angle += tri.ang_vel * dt;

        if tri.pos.y - tri.r < SIM_BOUNDS.y.start {
            tri.pos.y += (SIM_BOUNDS.y.start - (tri.pos.y - tri.r)) * 2.0;
            tri.vel.y *= -1.0;
        } else if tri.pos.y + tri.r > SIM_BOUNDS.y.end {
            tri.pos.y -= ((tri.pos.y + tri.r) - SIM_BOUNDS.y.end) * 2.0;
            tri.vel.y *= -1.0;
        }
        if tri.pos.x - tri.r < SIM_BOUNDS.x.start {
            tri.pos.x += (SIM_BOUNDS.x.start - (tri.pos.x - tri.r)) * 2.0;
            tri.vel.x *= -1.0;
        } else if tri.pos.x + tri.r > SIM_BOUNDS.x.end {
            tri.pos.x -= ((tri.pos.x + tri.r) - SIM_BOUNDS.x.end) * 2.0;
            tri.vel.x *= -1.0;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    if app.elapsed_frames() == 1 {
        frame.clear(nannou::color::named::WHITE);
    }

    let draw = app.draw();
    if !app.keys.down.is_empty() {
        draw.background().color(rgb8(255, 255, 255));
    }

    let m = app.mouse.position();
    draw.ellipse().w_h(40.0, 40.0).xy(m).finish();

    //let draw = draw.scale(m.x - win.x.start);
    let draw = draw.scale(697.0);

    let w = model.image.width() as f32;
    let h = model.image.height() as f32;

    for tri in &model.triangles {
        let in_0_1 = (tri.pos - SIM_BOUNDS.bottom_left()) / SIM_BOUNDS.wh();
        let color = model
            .image
            .get_pixel((in_0_1.x * w) as u32, ((1.0 - in_0_1.y) * h) as u32);

        draw.translate(tri.pos.into())
            .rotate(tri.angle.into())
            .tri()
            .points(
                Point2::new(-tri.r, -tri.r),
                Point2::new(tri.r, -tri.r),
                Point2::new(0.0, tri.r),
            )
            .color(rgba(color.0[0], color.0[1], color.0[2], 255));
    }
    draw.to_frame(app, &frame).unwrap();
    frame.submit();
}
