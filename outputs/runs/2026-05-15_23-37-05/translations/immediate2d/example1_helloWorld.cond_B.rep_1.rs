// Assuming immediate2d.rs is the equivalent Rust module of immediate2d.h

mod immediate2d {
    pub const YELLOW: u32 = 0xFFFFFF00; // Example color code

    pub fn draw_pixel(x: u32, y: u32, color: u32) {
        // This function would contain the logic to draw a pixel on the screen
        // For example, it might interface with a graphics library to render the pixel.
        println!("Drawing pixel at ({}, {}) with color #{:08X}", x, y, color);
    }
}

fn run() {
    // Draw a single pixel to the middle of the screen.
    immediate2d::draw_pixel(80, 60, immediate2d::YELLOW);

    // Additional exercises:
    // 1. Change the color of the pixel
    // 2. Draw another pixel, three spaces to the right
    // 3. Draw a mouth for your smiley face (4-6 more pixels), just below the eyes
}

fn main() {
    run();
}