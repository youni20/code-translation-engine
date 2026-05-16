const IMM2D_IMPLEMENTATION: bool = true; // Stand-in for include
use std::thread::sleep;
use std::time::Duration;

// Placeholder functions to simulate immediate2d library behavior
fn use_anti_aliasing() {
    // Simulate turning on anti-aliasing
}

fn draw_circle(x: i32, y: i32, radius: i32, fill_color: &str, border_color: &str) {
    // Simulate drawing a circle with given parameters
}

fn wait(milliseconds: u64) {
    sleep(Duration::from_millis(milliseconds));
}

fn clear() {
    // Simulate clearing the screen
}

fn yellow() -> &'static str {
    "Yellow"
}

fn brown() -> &'static str {
    "Brown"
}

fn run() {
    let x = 80;
    let y = 60;
    let radius = 20;
    let delay = 1000;

    // Draw shapes with smooth edges
    use_anti_aliasing();

    // Loop forever
    loop {
        draw_circle(x, y, radius, yellow(), brown());
        wait(delay);

        clear();
        wait(delay);
    }
}

fn main() {
    run();
}