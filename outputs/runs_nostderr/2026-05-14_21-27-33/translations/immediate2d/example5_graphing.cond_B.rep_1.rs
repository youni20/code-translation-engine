const WIDTH: i32 = 800; // example value; set to your desired width
const HEIGHT: i32 = 600; // example value; set to your desired height

fn f(x: f64) -> f64 {
    x.sin()
}

fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, color: &str) {
    // This is a placeholder for actual drawing line function
    println!("Drawing line from ({}, {}) to ({}, {}) with thickness {} and color {}", x1, y1, x2, y2, thickness, color);
}

fn draw_pixel(x: i32, y: i32, color: &str) {
    // This is a placeholder for actual drawing pixel function
    println!("Drawing pixel at ({}, {}) with color {}", x, y, color);
}

fn run() {
    // Draw our axes
    draw_line(0, HEIGHT / 2, WIDTH, HEIGHT / 2, 1, "DarkGray");
    draw_line(WIDTH / 2, 0, WIDTH / 2, HEIGHT, 1, "DarkGray");

    for p in 0..WIDTH {
        // We'll need to transform from our pixel coordinate system to the usual Cartesian coordinates
        let mut x = p as f64;

        // First, shift x=0 to the center of the screen (instead of the left edge)
        x -= (WIDTH / 2) as f64;

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
        y += (HEIGHT / 2) as f64;

        draw_pixel(p, y as i32, "LightRed");
    }
}

fn main() {
    run();
}