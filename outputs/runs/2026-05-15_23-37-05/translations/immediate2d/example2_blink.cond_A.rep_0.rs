// Assume that `immediate2d.h` has equivalent functionality in Rust, which is implemented here.

mod immediate2d {
    use std::thread::sleep;
    use std::time::Duration;
    
    pub enum Color {
        Yellow,
        Brown,
    }

    pub fn use_anti_aliasing() {
        // Implementation details for anti-aliasing setup
    }

    pub fn draw_circle(x: i32, y: i32, radius: i32, fill: Color, outline: Color) {
        // Implementation details for drawing a circle
    }

    pub fn clear() {
        // Implementation details for clearing the screen
    }

    pub fn wait(millis: u64) {
        sleep(Duration::from_millis(millis));
    }
}

fn run() {
    let x = 80;
    let y = 60;
    let radius = 20;
    let delay = 1000;

    // Draw shapes with smooth edges
    immediate2d::use_anti_aliasing();

    // Loop forever
    loop {
        immediate2d::draw_circle(x, y, radius, immediate2d::Color::Yellow, immediate2d::Color::Brown);
        immediate2d::wait(delay);

        immediate2d::clear();
        immediate2d::wait(delay);
    }
}

fn main() {
    run();
}