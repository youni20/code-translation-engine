// Assuming the necessary graphics library functions and type definitions are defined here
// For the example, we define fake DrawPixel and Color for demonstration purposes 

// Define Color as an enum to simulate the behavior
enum Color {
    Yellow,
}

// Define a mock DrawPixel function
fn draw_pixel(x: u32, y: u32, color: Color) {
    // Example function body that does nothing
    match color {
        Color::Yellow => println!("Drawing pixel at ({}, {}) with color Yellow", x, y),
    }
}

fn main() {
    draw_pixel(80, 60, Color::Yellow);
}