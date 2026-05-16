// Assume immediate2d module with a similar interface exists in Rust

mod immediate2d {
    pub fn draw_pixel(_x: u32, _y: u32, _color: Color) {
        // Implementation that draws a pixel on the screen
    }

    #[derive(Copy, Clone)]
    pub enum Color {
        Yellow,
        // ... other colors
    }
}

use immediate2d::{draw_pixel, Color};

fn main() {
    run();
}

fn run() {
    draw_pixel(80, 60, Color::Yellow);
}