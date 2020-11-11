use nannou::prelude::*;

struct Model {
    centers: Vec<Vec<Vector2>>,
    angles: Vec<Vec<f32>>,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

const N: usize = 12;

fn model(_app: &App) -> Model {
    let mut result = Model {
        centers: vec![],
        angles: vec![],
    };

    let n = (N - 1) as f32;

    for y in 0..N {
        result.centers.push(vec![]);
        result.angles.push(vec![]);
        for x in 0..N {
            result.centers[y].push(Vector2::new(x as f32 / n, y as f32 / n));
            //result.angles[y].push(thread_rng().gen::<f32>() * 2.0 * PI);
            result.angles[y].push(0.0);
        }
    }
    result
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(app, model, upd),
        _ => (),
    }
}

fn update(_app: &App, model: &mut Model, upd: Update) {
    let dt = upd.since_last.as_secs_f32();
    for (y, angles) in model.angles.iter_mut().enumerate() {
        for (x, angle) in angles.iter_mut().enumerate() {
            *angle += 0.005
                + 0.02 * (PI * y as f32 / N as f32 + dt).sin()
                + 0.02 * (PI * x as f32 / N as f32 - dt).cos();
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(rgb8(238, 168, 0));
    let win = app.window_rect();
    let draw = app.draw();
    let draw = draw.translate(Vector3::new(-win.x.len() / 2.0, -win.y.len() / 2.0, 0.0));

    let pt = |i: usize, j: usize| {
        let ang: f32 = model.angles[i][j];
        model.centers[i][j] * win.top_right() * 2.0 + Vector2::new(ang.cos(), ang.sin()) * 30.0
    };

    for i in 0..(N - 1) {
        for j in 0..(N - 1) {
            draw.tri()
                .points(pt(i, j), pt(i + 1, j), pt(i, j + 1))
                .color(rgb8(197, 50, 0));
        }
    }

    draw.to_frame(app, &frame).unwrap();
    frame.submit();
}
