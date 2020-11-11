use nannou::prelude::*;

struct Model {}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    Model {}
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(app, model, upd),
        _ => (),
    }
}

fn update(_app: &App, _model: &mut Model, _upd: Update) {}

const N: i32 = 10;

fn view(app: &App, _model: &Model, frame: Frame) {
    if app.elapsed_frames() == 1 {
        frame.clear(nannou::color::named::WHITE);
    }
    frame.clear(rgb8(71, 59, 240));

    let win = app.window_rect();
    let draw = app.draw();

    let draw = draw.scale(win.x.len()).translate(vec3(-0.5, -0.5, 0.0));

    for i in 0..N {
        for j in 0..N {
            let a = (i as f32) / ((N - 1) as f32);
            let b = (j as f32) / ((N - 1) as f32);

            let w_base = 1.0 / N as f32;
            let t = app.duration.since_start.as_secs_f32();

            //let f = ((t + a + b * t) * 0.7).sin();
            let f = ((t + a - b) * 0.7).sin();
            let w = w_base * f.abs();
            draw.ellipse()
                .resolution(32)
                .x_y(a, b)
                .w_h(w, w)
                .color(rgba(0.5, 1.0, 0.0, 1.0 - f.abs()));
        }
    }

    draw.to_frame(app, &frame).unwrap();
    frame.submit();
}
