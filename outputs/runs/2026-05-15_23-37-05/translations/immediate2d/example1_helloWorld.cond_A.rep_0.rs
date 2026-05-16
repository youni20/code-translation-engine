// Let's assume the immediate2d module is mimicked with appropriate Rust functions and constants.

mod immediate2d {
    #[derive(Debug)]
    pub enum Color {
        Yellow,
        // Other colors would be listed here if necessary.
    }

    pub fn draw_pixel(x: i32, y: i32, color: Color) {
        // Placeholder implementation for drawing a pixel.
        println!("Drawing pixel at ({}, {}) with color {:?}", x, y, color);
    }
}

use immediate2d::{draw_pixel, Color};

fn run() {
    draw_pixel(80, 60, Color::Yellow);
}

fn main() {
    run();
}