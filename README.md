# nannou sketches
A selection of interactive art that I've created. Made using the Rust programming language and the [Nannou](https://github.com/nannou-org/nannou) framework.

Each sketch is "example" of the cargo project. To build them, you'll need rust [Rust](https://www.rust-lang.org/). You'll also need to install Nannou's [platform-specific dependencies](https://guide.nannou.cc/getting_started/platform-specific_setup.html) for your platform.

Then, `cd` to the root of this project, and run:
- `cargo run --example [name]`

On the first run, this will take ~5 minutes to download and build Nannou for your platform. Building any other sketch after should be much faster.

Available sketches:
- `bouncing_1`, `bouncing_2`, `bouncing_3`: pieces based on objects / springs bouncing with Newtonian physics. Move your mouse left/right to adjust zoom.
- `pattern_1`, `pattern_2`, `pattern_3`: non-interactive sine-wave-based patterns.
- `poi`: a digital [poi](https://en.wikipedia.org/wiki/Poi_(performance_art)). Try moving your mouse around!
- `ripple_carry_circuit`: a visualization of a binary ripple-carry adder. Click the nodes on the left to change the inputs, and watch how they propagate through to the outputs.

Project layout:
Each sketch has its own file in the `examples` folder.
`src/` contains shared support code; currently the only relevant file is `src/circuits.rs`, which implements a simple digital circuit simulation + tests.

