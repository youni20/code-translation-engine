fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color) {
    // Placeholder function implementation
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Placeholder function implementation
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _width: i32, _color: Color) {
    // Placeholder function implementation
}

fn wait(_ms: u32) {
    // Placeholder function implementation
}

fn last_key() -> char {
    // Placeholder function implementation
    ' '
}

fn mouse_x() -> i32 {
    // Placeholder function implementation
    0
}

fn mouse_y() -> i32 {
    // Placeholder function implementation
    0
}

fn left_mouse_pressed() -> bool {
    // Placeholder function implementation
    false
}

#[derive(Clone, Copy)]
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

const WIDTH: i32 = 800; // Example width
const HEIGHT: i32 = 600; // Example height

fn run() {
    // Add the built-in colors to a list so we can retrieve them by array index
    let colors = [
        Color::Black,
        Color::Blue,
        Color::Green,
        Color::Cyan,
        Color::Red,
        Color::Magenta,
        Color::Brown,
        Color::LightGray,
        Color::DarkGray,
        Color::LightBlue,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightRed,
        Color::LightMagenta,
        Color::Yellow,
        Color::White,
    ];

    const PALETTE_ENTRY_WIDTH: i32 = WIDTH / 8;
    const PALETTE_ENTRY_HEIGHT: i32 = 6;

    // The canvas begins at three "rows" of palette down: the palette really only contains two rows,
    // but the third is used to show the currently selected color. The extra +2 is to leave a space
    // between the palette area and the canvas area.
    const CANVAS_Y: i32 = PALETTE_ENTRY_HEIGHT * 3 + 2;

    // Draw the palette at the top of the screen
    for i in 0..16 {
        // Halfway through our loop, wrap x back to the first column (using the "remainder" or "modulus" operator)
        let x = PALETTE_ENTRY_WIDTH * (i % 8);
        let mut y = 0;

        // Halfway through our loop, start a second row of colors
        if i >= 8 {
            y += PALETTE_ENTRY_HEIGHT;
        }

        draw_rectangle(x, y, PALETTE_ENTRY_WIDTH, PALETTE_ENTRY_HEIGHT, colors[i as usize]);
    }

    // This holds the index to the currently selected color (from the "colors" list).
    // The default is White, which is the 15th (zero-based) entry in the list.
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
            draw_rectangle(0, CANVAS_Y, WIDTH, HEIGHT, Color::Black);
        }

        // Grab our current mouse coordinates
        let x = mouse_x();
        let y = mouse_y();

        let mouse_offscreen = x < 0 || y < 0;
        let mouse_on_canvas = !mouse_offscreen && y > CANVAS_Y;

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
        if y < PALETTE_ENTRY_HEIGHT * 2 {
            // Determine which color was clicked
            let row = if y > PALETTE_ENTRY_HEIGHT { 1 } else { 0 };

            let col = x * 8 / WIDTH;

            // Calculate the new index into our "colors" table
            current_color = row * 8 + col;

            // Show the new color on the screen
            draw_rectangle(0, PALETTE_ENTRY_HEIGHT * 2 + 1, WIDTH, PALETTE_ENTRY_HEIGHT, colors[current_color as usize]);
        }

        // The rest of the loop is for drawing on the canvas, which we only
        // do if we're currently hovering over the canvas area of the screen
        if !mouse_on_canvas {
            continue;
        }

        // If the mouse hasn't moved yet, keep the coordinates
        // for later and just draw a single pixel for now
        if previous_x == -1 && previous_y == -1 {
            draw_pixel(x, y, colors[current_color as usize]);
            previous_x = x;
            previous_y = y;
            continue;
        }

        // Draw the line!
        draw_line(previous_x, previous_y, x, y, 1, colors[current_color as usize]);

        // Keep track of our new end-point for next time
        previous_x = x;
        previous_y = y;
    }
}

fn main() {
    run();
}