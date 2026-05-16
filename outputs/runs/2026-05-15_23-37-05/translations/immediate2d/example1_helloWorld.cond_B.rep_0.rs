mod immediate2d {
    pub use colors::*;
    pub use operations::*;

    mod colors {
        pub const YELLOW: u32 = 0xFFFF00; // Assuming 24-bit color: Yellow
        // Additional color definitions would go here.
    }

    mod operations {
        use crate::immediate2d::colors;

        pub fn draw_pixel(x: u32, y: u32, color: u32) {
            // Implementation to draw a pixel to the screen.
            // Placeholder implementation for translation.
            println!("Drawing pixel at ({}, {}) with color: #{:06X}", x, y, color);
        }
    }
}

fn run() {
    immediate2d::draw_pixel(80, 60, immediate2d::YELLOW);
}

fn main() {
    run();
}