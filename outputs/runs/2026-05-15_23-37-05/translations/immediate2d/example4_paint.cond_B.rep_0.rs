fn main() {
    run();
}

fn run() {
    // Add the built-in colors to a vector so we can retrieve them by index
    let colors = [
        Color::Black, Color::Blue, Color::Green, Color::Cyan, Color::Red,
        Color::Magenta, Color::Brown, Color::LightGray, Color::DarkGray,
        Color::LightBlue, Color::LightGreen, Color::LightCyan, Color::LightRed,
        Color::LightMagenta, Color::Yellow, Color::White,
    ];

    let palette_entry_width = WIDTH / 8;
    let palette_entry_height = 6;

    // The canvas begins at three "rows" of palette down
    let canvas_y = palette_entry_height * 3 + 2;

    // Draw the palette at the top of the screen
    for i in 0..16 {
        // Halfway through our loop, wrap x back to the first column (using the "remainder" or "modulus" operator)
        let x = palette_entry_width * (i % 8);
        let mut y = 0;

        // Halfway through our loop, start a second row of colors
        if i >= 8 {
            y += palette_entry_height;
        }

        draw_rectangle(x, y, palette_entry_width, palette_entry_height, colors[i as usize].clone());
    }

    // This holds the index to the currently selected color (from the "colors" list).
    let mut current_color = 15;

    // These hold where we last saw the mouse on the canvas, or (-1, -1) if the last click was not on the canvas
    let mut previous_x = -1;
    let mut previous_y = -1;

    loop {
        // Adding a short wait between "frames" is a good idea so the CPU doesn't max out at 100%
        wait(1);

        let key = last_key();

        // The spacebar clears the canvas
        if key == ' ' {
            draw_rectangle(0, canvas_y, WIDTH, HEIGHT, colors[15].clone());
        }

        // Grab our current mouse coordinates
        let x = mouse_x();
        let y = mouse_y();

        let mouse_offscreen = x < 0 || y < 0;
        let mouse_on_canvas = !mouse_offscreen && y > canvas_y;

        // Unless the mouse is currently on the canvas with the button held, force a new line segment
        if !mouse_on_canvas || !left_mouse_pressed() {
            previous_x = -1;
            previous_y = -1;
        }

        // Wait until a mouse button is pressed before proceeding
        if !left_mouse_pressed() {
            continue;
        }

        // Is this click in the palette area?
        if y < palette_entry_height * 2 {
            // Determine which color was clicked
            let row = if y > palette_entry_height { 1 } else { 0 };
            let col = if WIDTH != 0 { x * 8 / WIDTH } else { 0 };

            // Calculate the new index into our "colors" table
            current_color = row * 8 + col;

            // Show the new color on the screen
            draw_rectangle(0, palette_entry_height * 2 + 1, WIDTH, palette_entry_height, colors[current_color as usize].clone());
        }

        // The rest of the loop is for drawing on the canvas, which we only
        // do if we're currently hovering over the canvas area of the screen
        if !mouse_on_canvas {
            continue;
        }

        // If the mouse hasn't moved yet, keep the coordinates
        // for later and just draw a single pixel for now
        if previous_x == -1 && previous_y == -1 {
            draw_pixel(x, y, colors[current_color as usize].clone());
            previous_x = x;
            previous_y = y;
            continue;
        }

        // Draw the line!
        draw_line(previous_x, previous_y, x, y, 1, colors[current_color as usize].clone());

        // Keep track of our new end-point for next time
        previous_x = x;
        previous_y = y;
    }
}

// Assumed API functions and types
// These declarations are placeholders and must be replaced with actual implementations
const WIDTH: i32 = 1; // Placeholder changed to a non-zero value
const HEIGHT: i32 = 1; // Placeholder

#[derive(Clone)]
enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White,
}

fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color) {
    // Placeholder for actual function implementation
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Placeholder for actual function implementation
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {
    // Placeholder for actual function implementation
}

fn wait(_milliseconds: u32) {
    // Placeholder for actual function implementation
}

fn last_key() -> char {
    // Placeholder for actual function implementation
    ' ' // Dummy return value
}

fn mouse_x() -> i32 {
    // Placeholder for actual function implementation
    0 // Dummy return value
}

fn mouse_y() -> i32 {
    // Placeholder for actual function implementation
    0 // Dummy return value
}

fn left_mouse_pressed() -> bool {
    // Placeholder for actual function implementation
    false // Dummy return value
}