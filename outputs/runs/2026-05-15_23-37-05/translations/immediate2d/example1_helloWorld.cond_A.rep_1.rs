mod immediate2d {
    // Assuming the equivalent of `immediate2d.h` methods and constants are defined here
    pub const YELLOW: Color = Color::new(255, 255, 0, 255);

    #[derive(Copy, Clone)]
    pub struct Color {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8,
    }

    impl Color {
        pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
            Color { r, g, b, a }
        }
    }

    pub fn draw_pixel(x: u32, y: u32, color: Color) {
        // Implementation that draws a pixel using the given color
    }
}

use immediate2d::*;

fn run() {
    draw_pixel(80, 60, YELLOW);
}

fn main() {
    run();
}