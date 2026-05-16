fn draw_pixel(x: u32, y: u32, color: &str) {
    // This function would interface with the graphics library to draw a pixel.
    // The actual implementation would depend on how the immediate2d.h library functions are ported/made available in Rust.
    println!("Drawing pixel at ({}, {}) with color: {}", x, y, color);
}

fn run() {
    draw_pixel(80, 60, "Yellow");
}

fn main() {
    run();
}