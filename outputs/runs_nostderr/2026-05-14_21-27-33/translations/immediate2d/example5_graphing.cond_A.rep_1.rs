use std::f64::consts::PI;

fn f(x: f64) -> f64 {
    x.sin()
}

fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, color: (u8, u8, u8)) {
    // Placeholder function for DrawLine
}

fn draw_pixel(x: i32, y: i32, color: (u8, u8, u8)) {
    // Placeholder function for DrawPixel
}

fn run() {
    const WIDTH: i32 = 800; // Placeholder value for Width
    const HEIGHT: i32 = 600; // Placeholder value for Height
    const DARK_GRAY: (u8, u8, u8) = (169, 169, 169);
    const LIGHT_RED: (u8, u8, u8) = (255, 182, 193);

    // Draw our axes
    draw_line(0, HEIGHT / 2, WIDTH, HEIGHT / 2, 1, DARK_GRAY);
    draw_line(WIDTH / 2, 0, WIDTH / 2, HEIGHT, 1, DARK_GRAY);

    for p in 0..WIDTH {
        // Transform from pixel coordinate system to Cartesian coordinates
        let mut x = p as f64;

        // Shift x=0 to the center of the screen
        x -= (WIDTH / 2) as f64;

        // Scale x down
        x /= 10.0;

        // Run the function
        let mut y = f(x);

        // Scale result back up
        y *= 10.0;

        // Adjust for coordinate system difference
        y = -y;

        // Shift y to the center of the screen
        y += (HEIGHT / 2) as f64;

        draw_pixel(p, y as i32, LIGHT_RED);
    }
}

fn main() {
    run();
}