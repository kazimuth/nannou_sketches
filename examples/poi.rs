use nannou::color::Lab;
use nannou::prelude::*;

const K: f32 = 30.0;
const EQUILIBRIUM: f32 = 60.0;
const GRAVITY: Vector2 = Vector2 {
    x: 0.0,
    y: -100000.0,
};

struct Model {
    pos: Vector2,
    vel: Vector2,
    mass: f32,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    Model {
        pos: vec2(0.0, 0.0),
        vel: vec2(0.0, 0.0),
        mass: 1.0,
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(app, model, upd),
        _ => (),
    }
}

fn update(app: &App, model: &mut Model, upd: Update) {
    if app.elapsed_frames() < 10 || !app.keys.down.is_empty() {
        model.pos = app.mouse.position() + vec2(-50.0, 0.1);
        model.vel = vec2(0.0, 0.0);
    }
    let dt = upd.since_last.as_secs_f32();

    let to_mouse = app.mouse.position() - model.pos;

    let spring = to_mouse.normalize() * (to_mouse.magnitude() - EQUILIBRIUM) * K;
    let gravity = GRAVITY * dt;
    let f = spring + gravity;
    let a = f / model.mass;
    model.vel += a * dt;
    model.vel *= 0.99;
    model.pos += model.vel * dt;
}

fn view(app: &App, model: &Model, frame: Frame) {
    if app.elapsed_frames() == 1 || !app.keys.down.is_empty() {
        frame.clear(nannou::color::named::WHITE);
    }

    let win = app.window_rect();
    let draw = app.draw();

    /*
    let step_ = |pos| {
        draw.rect()
            .x_y(0.0, 0.0)
            .w_h(win.x.len(), win.y.len())
            .color(rgba8(255, 255, 255, 1))
            .finish();

        draw.line()
            .start(app.mouse.position())
            .end(pos)
            .color(rgba8(0, 0, 0, 50))
            .finish();

        draw.ellipse()
            .xy(pos)
            .w_h(10.0, 10.0)
            .color(rgb8(0, 0, 0))
            .finish();
    };

    step_(model.pos - model.vel * (0.5 * 1.0 / app.fps()));
    step_(model.pos);
     */

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(win.x.len(), win.y.len())
        .color(rgba8(255, 255, 255, 1))
        .finish();

    draw.line()
        .start(app.mouse.position())
        .end(model.pos)
        .color(rgba8(0, 0, 0, 50))
        .finish();

    let color_a: Lab = rgb8(249, 0, 229).into_format::<f32>().into();
    let color_b: Lab = rgb8(0, 110, 255).into_format::<f32>().into();
    // 1/2 m v^2
    let kinetic = 0.5 * model.mass * model.vel.magnitude2();
    // 1/2 k d^2
    let potential = 0.5 * K * (model.pos - app.mouse.position()).magnitude2();
    let ratio = kinetic / (kinetic + potential);
    let color = color_a * ratio + color_b * (1.0 - ratio);
    let color = Rgb::from(color).into_format::<u8>();

    draw.ellipse()
        .xy(app.mouse.position())
        .w_h(2.0, 2.0)
        .color(rgb8(0, 0, 0))
        .finish();

    draw.line()
        .start(model.pos)
        .end(model.pos - (model.vel * (1.0 / app.fps())))
        .weight(10.0)
        .caps_round()
        .color(color)
        .finish();

    draw.to_frame(app, &frame).unwrap();

    frame.submit();
}
