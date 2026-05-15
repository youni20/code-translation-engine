mod immediate2d {
    // Assume that immediate2d.h defines color constants and a function like:
    // pub const YELLOW: Color = ...;
    // pub fn draw_pixel(x: i32, y: i32, color: Color);
    // Here is a Rust equivalent

    pub struct Color {
        // Representation of Color
    }

    pub const YELLOW: Color = Color {};
    
    pub fn draw_pixel(x: i32, y: i32, color: &Color) {
        // Function implementation to draw a pixel
    }
}

// Bringing the necessary components into scope
use immediate2d::{draw_pixel, YELLOW};

fn run() {
    draw_pixel(80, 60, &YELLOW);
}

fn main() {
    run();
}