use nannou::color::Lab;
use nannou::geom::Range;
use nannou::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

const FOR_RENDER: bool = false;
const RENDER_DT: f32 = 1.0 / 60.0;
const RENDER_START: u64 = 240;
const RENDER_END: u64 = 620;

#[derive(Debug)]
struct Ball {
    pos: Vector2<f32>,
    prev_pos: Vector2<f32>,
    vel: Vector2<f32>,
    r: f32,
    color: Rgb8,
}

struct Model {
    balls: Vec<Ball>,
}

const N: u32 = 30;
const GRAVITY: Vector2<f32> = Vector2 { x: 0.0, y: -1.0 };

// domain is (-.5, .5) x (-.5, .5)
const SIM_BOUNDS: Rect<f32> = Rect {
    x: Range {
        start: -0.5,
        end: 0.5,
    },
    y: Range {
        start: -0.5,
        end: 10.,
    },
};

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let color_a: Lab = rgb8(249, 0, 229).into_format::<f32>().into();
    let color_b: Lab = rgb8(0, 110, 255).into_format::<f32>().into();

    let lerp = |a: f32| {
        let result = color_a * a + color_b * (1.0 - a);
        let result: Rgb = result.into();
        result.into_format::<u8>()
    };

    let mut rng: XorShiftRng = SeedableRng::seed_from_u64(12345);
    let balls = (0..N)
        .map(|_| {
            let pos = rng.gen::<Vector2<f32>>() - vec2(0.5f32, 0.5f32);
            Ball {
                pos,
                prev_pos: pos,
                vel: rng.gen::<Vector2<f32>>() - vec2(0.5f32, 0.5f32),
                r: rng.gen::<f32>() / 30.0 + 0.02,
                color: lerp(rng.gen()),
            }
        })
        .collect::<Vec<_>>();

    Model { balls }
}

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(model, upd),
        _ => (),
    }
}

fn update(model: &mut Model, upd: Update) {
    let dt = if FOR_RENDER {
        RENDER_DT
    } else {
        upd.since_last.as_secs_f32()
    };

    for ball in model.balls.iter_mut() {
        ball.prev_pos = ball.pos;

        ball.vel += GRAVITY * dt;
        ball.pos += ball.vel * dt;

        if ball.pos.y - ball.r < SIM_BOUNDS.y.start {
            ball.pos.y += (SIM_BOUNDS.y.start - (ball.pos.y - ball.r)) * 2.0;
            ball.vel.y *= -1.0;
        }
        if ball.pos.x - ball.r < SIM_BOUNDS.x.start {
            ball.pos.x += (SIM_BOUNDS.x.start - (ball.pos.x - ball.r)) * 2.0;
            ball.vel.x *= -1.0;
        } else if ball.pos.x + ball.r > SIM_BOUNDS.x.end {
            ball.pos.x -= ((ball.pos.x + ball.r) - SIM_BOUNDS.x.end) * 2.0;
            ball.vel.x *= -1.0;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    if app.elapsed_frames() == 1 {
        frame.clear(nannou::color::named::WHITE);
    }
    let win = app.window_rect();
    let draw = app.draw();
    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(win.x.len(), win.y.len())
        .color(rgba8(255, 255, 255, 5))
        .finish();

    let m = app.mouse.position();

    let draw = if FOR_RENDER {
        draw.scale(745.0)
    } else {
        draw.scale(m.x - win.x.start)
    };

    let color_a: Lab = rgb8(249, 0, 229).into_format::<f32>().into();
    let color_b: Lab = rgb8(0, 110, 255).into_format::<f32>().into();

    for ball in &model.balls {
        // 1/2 m v^2
        let kinetic = 0.5 * ball.vel.magnitude2();
        // m g h
        let potential = GRAVITY.magnitude() * ((ball.pos.y - ball.r) - SIM_BOUNDS.y.start);

        let ratio = potential / (potential + kinetic);

        draw.line()
            .start(ball.prev_pos)
            .end(ball.pos)
            .weight(ball.r)
            .caps_round()
            .tolerance(0.001)
            .color(color_a * ratio + (color_b * (1.0 - ratio)))
            .finish();
    }
    draw.to_frame(app, &frame).unwrap();
    if FOR_RENDER && frame.nth() >= RENDER_START && frame.nth() < RENDER_END {
        // Capture the frame!
        let file_path = captured_frame_path(app, &frame);
        app.main_window().capture_frame(file_path);
    }
    frame.submit();
}

fn captured_frame_path(app: &App, frame: &Frame) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `/<path_to_nannou>/nannou/simple_capture`.
        .join(app.exe_name().unwrap())
        // Name each file after the number of the frame.
        .join(format!("{:03}", frame.nth() - RENDER_START))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}
