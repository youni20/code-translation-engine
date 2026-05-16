const IMM2D_IMPLEMENTATION: bool = true;

// Placeholder for the drawing functions and constants
const Height: i32 = 480;
const Width: i32 = 640;
const DarkGray: u32 = 0xA9A9A9;
const LightRed: u32 = 0xFFCCCB;

/// Placeholder function for drawing a line
fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, _width: i32, _color: u32) {
    // Implementation of line drawing
    println!("Drawing line from ({}, {}) to ({}, {})", x1, y1, x2, y2);
}

/// Placeholder function for drawing a pixel
fn draw_pixel(x: i32, y: i32, _color: u32) {
    // Implementation of pixel drawing
    println!("Drawing pixel at ({}, {})", x, y);
}

/// Math function `f` equivalent to `sin(x)`
fn f(x: f64) -> f64 {
    x.sin()
}

fn run() {
    // Draw our axes
    draw_line(0, Height / 2, Width, Height / 2, 1, DarkGray);
    draw_line(Width / 2, 0, Width / 2, Height, 1, DarkGray);

    for p in 0..Width {
        // Transform from pixel coordinate system to Cartesian coordinate
        let mut x = p as f64;

        // Shift x=0 to the center
        x -= Width as f64 / 2.0;

        // Scale down for setting graph's window
        x /= 10.0;

        // Run the function!
        let mut y = f(x);

        // Scale result back up
        y *= 10.0;

        // Flip y for Cartesian system and center on the screen
        y = -y + Height as f64 / 2.0;

        draw_pixel(p, y as i32, LightRed);
    }
}

fn main() {
    run();
}