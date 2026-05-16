const HEIGHT: i32 = 480; // Example value, should match your actual height.
const WIDTH: i32 = 640; // Example value, should match your actual width.

fn f(x: f64) -> f64 {
    x.sin()
}

fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, width: i32, color: u32) {
    // Implementation needed for drawing a line, similar to DrawLine in C++.
}

fn draw_pixel(x: i32, y: i32, color: u32) {
    // Implementation needed for drawing a pixel, similar to DrawPixel in C++.
}

fn run() {
    const DARK_GRAY: u32 = 0xA9A9A9; // Example color value
    const LIGHT_RED: u32 = 0xFFCCCB; // Example color value

    // Draw our axes
    draw_line(0, HEIGHT / 2, WIDTH, HEIGHT / 2, 1, DARK_GRAY);
    draw_line(WIDTH / 2, 0, WIDTH / 2, HEIGHT, 1, DARK_GRAY);

    for p in 0..WIDTH {
        // We'll need to transform from our pixel coordinate system to the usual Cartesian coordinates
        let mut x = p as f64;

        // First, shift x=0 to the center of the screen (instead of the left edge)
        x -= WIDTH as f64 / 2.0;

        // Scale x down, effectively setting our graph's "Window" to something like w=(-8, 8), h=(-6, 6)
        x /= 10.0;

        // Run the function!
        let mut y = f(x);

        // Now we have to scale our result back up
        y *= 10.0;

        // In computer graphics, y increases downward
        // In Cartesian coordinates, y increases upward
        y = -y;

        // Finally, shift it to the center of the screen (instead of the top edge)
        y += HEIGHT as f64 / 2.0;

        draw_pixel(p, y as i32, LIGHT_RED);
    }
}

fn main() {
    run();
}