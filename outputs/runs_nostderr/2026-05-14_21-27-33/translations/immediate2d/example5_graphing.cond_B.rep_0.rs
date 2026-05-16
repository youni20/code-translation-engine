fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _width: i32, _color: u32) {
    // Implementation to draw a line, placeholder function
}

fn draw_pixel(_x: i32, _y: i32, _color: u32) {
    // Implementation to draw a pixel, placeholder function
}

const WIDTH: i32 = 800; // Example placeholder value for width
const HEIGHT: i32 = 600; // Example placeholder value for height
const LIGHT_RED: u32 = 0xFFF0E0E0; // Example color value
const DARK_GRAY: u32 = 0xFF404040; // Example color value

fn f(x: f64) -> f64 {
    x.sin()
}

fn run() {
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