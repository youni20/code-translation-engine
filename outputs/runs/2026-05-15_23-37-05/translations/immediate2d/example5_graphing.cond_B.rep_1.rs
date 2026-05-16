const WIDTH: i32 = 640; // Example width
const HEIGHT: i32 = 480; // Example height

fn f(x: f64) -> f64 {
    x.sin()
}

fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, color: Color) {
    // Placeholder for the line drawing function
}

fn draw_pixel(x: i32, y: i32, color: Color) {
    // Placeholder for the pixel drawing function
}

fn run() {
    // Draw axes
    draw_line(0, HEIGHT / 2, WIDTH, HEIGHT / 2, 1, Color::DarkGray);
    draw_line(WIDTH / 2, 0, WIDTH / 2, HEIGHT, 1, Color::DarkGray);

    for p in 0..WIDTH {
        // Transform from pixel coordinates to Cartesian coordinates
        let mut x = p as f64;

        // Shift x=0 to the center of the screen
        x -= (WIDTH / 2) as f64;

        // Scale x down
        x /= 10.0;

        // Run the function
        let mut y = f(x);

        // Scale the result back up
        y *= 10.0;

        // Invert y to switch coordinate systems
        y = -y;

        // Shift to the center of the screen
        y += (HEIGHT / 2) as f64;

        // Draw the pixel
        draw_pixel(p, y as i32, Color::LightRed);
    }
}

#[derive(Debug, Copy, Clone)]
enum Color {
    DarkGray,
    LightRed,
}

fn main() {
    run();
}