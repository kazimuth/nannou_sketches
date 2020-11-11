use nannou::prelude::*;
use nannou_sketches::circuits::*;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;

const N: usize = 8;
const K: f32 = 3.0;
const FRICTION: f32 = 0.96;
const GOAL_LENGTH: f32 = 1.0 / (N as f32);

const UPDATE_EVERY: f32 = 1.0 / 5.0;

const USE_SPRINGS: bool = false;

struct Model {
    circuit: Circuit,
    a: Vec<NodeIndex>,
    b: Vec<NodeIndex>,
    s: Vec<NodeIndex>,
    c: NodeIndex,

    positions: HashMap<NodeIndex, Vector2>,
    velocities: HashMap<NodeIndex, Vector2>,

    update_order: Vec<NodeIndex>,

    selected: NodeIndex,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let mut circuit = Circuit::new();

    let a = (0..N)
        .into_iter()
        .map(|_| circuit.add_input())
        .collect::<Vec<_>>();
    let b = (0..N)
        .into_iter()
        .map(|_| circuit.add_input())
        .collect::<Vec<_>>();
    let (s, c) = circuit.ripple_carry(&a, &b);
    let c = circuit.add_output(c);
    let s = s
        .into_iter()
        .map(|si| circuit.add_output(si))
        .collect::<Vec<_>>();

    let update_order = circuit.update_order();

    let mut ranks = circuit.ranks();
    let max = *ranks.values().max().unwrap();
    for s in &s {
        ranks.insert(*s, max + 1);
    }
    /*
    for (n, r) in ranks.iter_mut() {
        if *r == 2 && circuit.0[*n] == Gate::And || *r >= 3 {
            *r += 1;
        }
    }
     */

    let mut positions = HashMap::new();
    let mut velocities = HashMap::new();

    if !USE_SPRINGS {
        let flipped = flip_ranks(&ranks);
        let x_slots = (flipped.len()) as f32;
        for (i, rank) in flipped.iter().enumerate() {
            let y_slots = (rank.len()) as f32;

            for (j, node) in rank.iter().enumerate() {
                if circuit.0[*node] == Gate::MetaInput {
                    continue;
                }

                positions.insert(
                    *node,
                    vec2(
                        (i + 1) as f32 / (x_slots + 1.0),
                        1.0 - ((j + 1) as f32 / (y_slots + 1.0)),
                    ),
                );
            }
        }
    }
    for node in circuit.0.node_indices() {
        if USE_SPRINGS {
            positions.insert(node, nannou::rand::rand::random());
        }
        velocities.insert(node, vec2(0.0, 0.0));
    }
    for i in 0..N {
        positions.insert(a[i], vec2(0.0, 1.0 - (i as f32 / (N * 2) as f32)));
        positions.insert(b[i], vec2(0.0, 0.5 - (i as f32 / (N * 2) as f32)));
        positions.insert(s[i], vec2(1.0, 1.0 - (i as f32 / (N) as f32)));
    }
    positions.insert(c, vec2(1.0, 0.0));

    Model {
        circuit,
        a,
        b,
        s,
        c,
        positions,
        velocities,
        update_order,
        selected: c,
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::Update(upd) => update(app, model, upd),
        Event::WindowEvent {
            simple: Some(MousePressed(_)),
            ..
        } => {
            let current = model.circuit.get_1_in(model.selected);
            model.circuit.set_input(model.selected, !current);
        }
        Event::WindowEvent {
            simple:
                Some(Touch(TouchEvent {
                    phase: TouchPhase::Started,
                    position,
                    ..
                })),
            ..
        } => {
            let map_pos = make_map_pos(app.window_rect());
            let selected = *model
                .a
                .iter()
                .chain(model.b.iter())
                .min_by_key(|n| {
                    (((map_pos(model.positions[*n]) - position).magnitude2()) * 10000.0) as usize
                })
                .unwrap();

            let current = model.circuit.get_1_in(selected);
            model.circuit.set_input(selected, !current);
        }
        _ => (),
    }
}

fn epoch(t: f32) -> u32 {
    (t / UPDATE_EVERY).floor() as u32
}

fn update(app: &App, model: &mut Model, upd: Update) {
    let dt = upd.since_last.as_secs_f32();
    let t = app.duration.since_start.as_secs_f32();
    let map_pos = make_map_pos(app.window_rect());

    model.selected = *model
        .a
        .iter()
        .chain(model.b.iter())
        .min_by_key(|n| {
            (((map_pos(model.positions[*n]) - app.mouse.position()).magnitude2()) * 10000.0)
                as usize
        })
        .unwrap();

    if t < 0.2 || !app.keys.down.is_empty() {
        for i in 0..N {
            model.circuit.set_input(model.a[i], false);
            model.circuit.set_input(model.b[i], false);
        }
    }

    if epoch(t - dt) < epoch(t) {
        model.circuit.update_signals_once(&model.update_order);
    }

    if USE_SPRINGS && t < 30.0 {
        for node in model.circuit.0.node_indices() {
            let node_type = model.circuit.0[node];
            if node_type == Gate::MetaInput || node_type == Gate::Input || node_type == Gate::Output
            {
                continue;
            }
            let pos = model.positions[&node];
            let vel = model.velocities[&node];
            let mut force = vec2(0.0, 0.0);
            for edge in model.circuit.0.edges_directed(node, Direction::Incoming) {
                let d = model.positions[&edge.source()] - pos;
                force += d.normalize() * (d.magnitude() - GOAL_LENGTH) * K;
            }
            for edge in model.circuit.0.edges_directed(node, Direction::Outgoing) {
                let d = model.positions[&edge.target()] - pos;
                force += d.normalize() * (d.magnitude() - GOAL_LENGTH) * K;
            }
            force += vec2(1.0 - pos.x, 0.0);
            let vel = (vel + force * dt) * FRICTION;
            let pos = pos + vel * dt;
            model.positions.insert(node, pos);
            model.velocities.insert(node, vel);
        }
    }
    for (n, position) in model.positions.iter() {
        if position.x.is_nan() || position.y.is_nan() {
            println!("nan! {:?} {:?}", n, position);
        }
    }
}

static A_LABELS: &'static [&'static str] = &["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8"];
static B_LABELS: &'static [&'static str] = &["b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8"];
static S_LABELS: &'static [&'static str] = &["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8"];

fn make_map_pos(win: Rect) -> impl Fn(Vector2) -> Vector2 {
    let bl = win.bottom_left();
    let tr = win.top_right();
    let to_tr = tr - bl;
    let bl_ = bl + to_tr * 0.1;
    let to_tr_ = to_tr * 0.8;

    move |p: Vector2| bl_ + vec2(to_tr_.x * p.x, to_tr_.y * p.y)
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(rgb8(50, 50, 50));
    let win = app.window_rect();
    let draw = app.draw();
    let map_pos = make_map_pos(win);

    let edges = model.circuit.0.edge_count() as f32;

    for (i, edge) in model.circuit.0.edge_references().enumerate() {
        if model.circuit.0[edge.target()] == Gate::Input {
            continue;
        }
        let hue = (i as f32) / edges;
        let lightness = if *edge.weight() { 0.7 } else { 0.1 };
        let color = hsl(hue, 1.0, lightness);

        draw.line()
            .start(map_pos(model.positions[&edge.source()]))
            .end(map_pos(model.positions[&edge.target()]))
            .weight(5.0)
            .color(color);
    }

    for node in model.circuit.0.node_indices() {
        let text = match model.circuit.0[node] {
            Gate::MetaInput => continue,
            Gate::Input | Gate::Output => "",
            Gate::Or => "|",
            Gate::And => "&",
            Gate::Not => "!",
            Gate::Xor => "^",
        };
        let pos = map_pos(model.positions[&node]);
        let ellipse_color = if node == model.selected {
            rgb8(100, 100, 200)
        } else {
            rgb8(100, 100, 100)
        };
        draw.ellipse().xy(pos).w_h(20.0, 20.0).color(ellipse_color);

        draw.text(text).xy(pos).color(rgb8(255, 255, 255));
    }
    let (mut a_, mut b_, mut s_) = (0, 0, 0);
    for (i, a) in model.a.iter().enumerate() {
        draw.text(&A_LABELS[i])
            .xy(map_pos(model.positions[a]))
            .color(rgb8(255, 255, 255));

        a_ = set_bit(a_, i, model.circuit.get_1_in(*a));
    }
    for (i, b) in model.b.iter().enumerate() {
        draw.text(&B_LABELS[i])
            .xy(map_pos(model.positions[b]))
            .color(rgb8(255, 255, 255));
        b_ = set_bit(b_, i, model.circuit.get_1_in(*b));
    }
    for (i, s) in model.s.iter().enumerate() {
        draw.text(&S_LABELS[i])
            .xy(map_pos(model.positions[s]))
            .color(rgb8(255, 255, 255));
        s_ = set_bit(s_, i, model.circuit.get_1_in(*s));
    }
    s_ = set_bit(s_, 8, model.circuit.get_1_in(model.c));

    draw.text(&format!("{}", a_))
        .xy(map_pos(vec2(-0.07, 0.785)))
        .font_size(16);

    draw.text(&format!("{}", b_))
        .xy(map_pos(vec2(-0.07, 0.285)))
        .font_size(16);

    draw.text(&format!("{}", s_))
        .xy(map_pos(vec2(1.07, 0.5)))
        .font_size(16);

    draw.line()
        .start(map_pos(vec2(-0.05, 1.0 - 0.0 / (N as f32 * 2.0))))
        .end(map_pos(vec2(
            -0.05,
            1.0 - ((N - 1) as f32) / (N as f32 * 2.0),
        )))
        .color(rgb8(255, 255, 255));

    draw.line()
        .start(map_pos(vec2(-0.05, 0.5 - 0.0 / (N as f32 * 2.0))))
        .end(map_pos(vec2(
            -0.05,
            0.5 - ((N - 1) as f32) / (N as f32 * 2.0),
        )))
        .color(rgb8(255, 255, 255));

    draw.line()
        .start(map_pos(vec2(1.05, 1.0)))
        .end(map_pos(vec2(1.05, 0.0)))
        .color(rgb8(255, 255, 255));

    draw.text("^ click")
        .xy(map_pos(vec2(0.0, -0.05)))
        .color(rgb8(255, 255, 255))
        .font_size(16);

    draw.to_frame(app, &frame).unwrap();
    frame.submit();
}
