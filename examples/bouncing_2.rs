use nannou::color::Lab;
use nannou::geom::Range;
use nannou::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

#[derive(Debug)]
struct Ball {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    r: f32,
    color: Rgb8,
}

#[derive(Debug)]
struct Connection {
    equilibrium: f32,
    hooke: f32,
    a: usize,
    b: usize,
}

struct Model {
    balls: Vec<Ball>,
    connections: Vec<Connection>,
}

const N: usize = 30;
const M: usize = 20;
const FIXED: usize = 5;
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
        .map(|_| Ball {
            pos: rng.gen::<Vector2<f32>>() - vec2(0.5f32, 0.5f32),
            vel: rng.gen::<Vector2<f32>>() - vec2(0.5f32, 0.5f32),
            r: rng.gen::<f32>() / 30.0 + 0.02,
            color: lerp(rng.gen()),
        })
        .collect::<Vec<_>>();
    let connections = (0..M)
        .map(|_| {
            let a = rng.gen::<usize>() % N;
            let mut b = rng.gen::<usize>() % N;
            while b == a {
                b = rng.gen::<usize>() % N;
            }
            Connection {
                equilibrium: (balls[a].pos - balls[b].pos).magnitude() / 10.0,
                hooke: rng.gen::<f32>() / 2.0,
                a,
                b,
            }
        })
        .collect::<Vec<_>>();

    Model { balls, connections }
}

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(model, upd),
        _ => (),
    }
}

fn update(model: &mut Model, upd: Update) {
    let dt = upd.since_last.as_secs_f32();
    for (i, ball) in model.balls.iter_mut().enumerate() {
        if i < FIXED {
            continue;
        }
        ball.vel += GRAVITY * dt;
    }
    for connection in model.connections.iter() {
        let pa = model.balls[connection.a].pos;
        let pb = model.balls[connection.b].pos;
        let pa_to_pb = pb - pa;
        let length = pa_to_pb.magnitude2();
        let f = connection.hooke * (length - connection.equilibrium);
        let pa_to_pb_n = pa_to_pb / length;

        // TODO mass?
        model.balls[connection.a].vel += pa_to_pb_n * f;
        model.balls[connection.b].vel -= pa_to_pb_n * f;
    }

    for (i, ball) in model.balls.iter_mut().enumerate() {
        if i < FIXED {
            continue;
        }
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
        ball.vel *= 0.99;
    }

    /*
    for (i, ball1) in model.balls.iter_mut().enumerate() {
        for (j, ball2) in model.balls.iter_mut().enumerate() {
            if i == j {
                continue;
            }
        }
    }
     */
}

fn view(app: &App, model: &Model, frame: Frame) {
    if app.elapsed_frames() == 1 {
        frame.clear(nannou::color::named::WHITE);
    }
    let win = app.window_rect();
    let draw = app.draw();
    //draw.rect()
    //    .x_y(0.0, 0.0)
    //    .w_h(win.x.len(), win.y.len())
    //    .color(rgba8(255, 255, 255, 5))
    //    .finish();
    draw.background().color(rgb8(255, 255, 255));

    let m = app.mouse.position();
    //draw.text(&format!("[{:.2}, {:.2}]", m.x, m.y)).xy(m).finish();
    draw.text(&format!("{:.2}", m.x - win.x.start))
        .xy(m)
        .finish();

    let draw = draw.scale(m.x - win.x.start);
    //let draw = draw.scale(745.0);

    let color_a: Lab = rgb8(249, 0, 229).into_format::<f32>().into();
    let color_b: Lab = rgb8(0, 110, 255).into_format::<f32>().into();
    //let color_b: Lab = rgb8(0, 230, 10).into_format::<f32>().into();

    for connection in &model.connections {
        draw.line()
            .start(model.balls[connection.a].pos)
            .end(model.balls[connection.b].pos)
            .weight(0.01)
            .color(rgba8(0, 0, 0, 40))
            .finish();
    }
    for ball in &model.balls {
        // 1/2 m v^2
        let kinetic = 0.5 * ball.vel.magnitude2();
        // m g h
        let potential = GRAVITY.magnitude() * ((ball.pos.y - ball.r) - SIM_BOUNDS.y.start);

        let ratio = potential / (potential + kinetic);

        draw.ellipse()
            .xy(ball.pos)
            //.color(ball.color)
            //.color(rgb(r2, 0, 255 - r2))
            .color(color_a * ratio + (color_b * (1.0 - ratio)))
            .w_h(ball.r, ball.r)
            .resolution(16)
            .finish();
    }
    draw.to_frame(app, &frame).unwrap();
    frame.submit();
}
