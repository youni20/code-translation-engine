const IMM2D_IMPLEMENTATION: bool = true;
use std::f64::consts::PI;

// Assuming some module or library provides these functions
// These functions would usually interface with a graphics library
fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, color: Color) {
    // Implementation assumes a graphics library
}

fn draw_pixel(x: i32, y: i32, color: Color) {
    // Implementation assumes a graphics library
}

#[derive(Clone, Copy)]
enum Color {
    DarkGray,
    LightRed,
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn f(x: f64) -> f64 {
    x.sin()
}

fn run() {
    // Draw our axes
    draw_line(0, HEIGHT / 2, WIDTH, HEIGHT / 2, 1, Color::DarkGray);
    draw_line(WIDTH / 2, 0, WIDTH / 2, HEIGHT, 1, Color::DarkGray);

    for p in 0..WIDTH {
        // Transform from pixel coordinate system to Cartesian coordinates
        let mut x = p as f64;

        // Shift x=0 to the center of the screen
        x -= WIDTH as f64 / 2.0;

        // Scale x down
        x /= 10.0;

        // Run the function
        let mut y = f(x);

        // Scale y back up
        y *= 10.0;

        // Flip y to account for coordinate system difference
        y = -y;

        // Shift y to the center of the screen
        y += HEIGHT as f64 / 2.0;

        draw_pixel(p, y as i32, Color::LightRed);
    }
}

fn main() {
    run();
}