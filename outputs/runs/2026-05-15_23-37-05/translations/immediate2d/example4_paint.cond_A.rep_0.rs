const COLORS: [Color; 16] = [
    Color::Black, Color::Blue, Color::Green, Color::Cyan, Color::Red, Color::Magenta, Color::Brown, Color::LightGray,
    Color::DarkGray, Color::LightBlue, Color::LightGreen, Color::LightCyan, Color::LightRed, Color::LightMagenta,
    Color::Yellow, Color::White,
];

const PALETTE_ENTRY_WIDTH: i32 = WIDTH / 8;
const PALETTE_ENTRY_HEIGHT: i32 = 6;
const CANVAS_Y: i32 = PALETTE_ENTRY_HEIGHT * 3 + 2;

fn main() {
    run();
}

pub fn run() {
    let mut current_color: usize = 15;
    let mut previous_x: i32 = -1;
    let mut previous_y: i32 = -1;

    loop {
        wait(1);

        let key: Option<char> = last_key();
        
        // The spacebar clears the canvas
        if let Some(' ') = key {
            draw_rectangle(0, CANVAS_Y, WIDTH, HEIGHT, Color::Black);
        }

        let x: i32 = mouse_x();
        let y: i32 = mouse_y();

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
            let row = if y > PALETTE_ENTRY_HEIGHT { 1 } else { 0 };
            let col = x * 8 / WIDTH;
            current_color = (row * 8 + col) as usize;

            // Show the new color on the screen
            draw_rectangle(0, PALETTE_ENTRY_HEIGHT * 2 + 1, WIDTH, PALETTE_ENTRY_HEIGHT, COLORS[current_color]);
        }

        if !mouse_on_canvas {
            continue;
        }

        if previous_x == -1 && previous_y == -1 {
            draw_pixel(x, y, COLORS[current_color]);
            previous_x = x;
            previous_y = y;
            continue;
        }

        draw_line(previous_x, previous_y, x, y, 1, COLORS[current_color]);
        
        previous_x = x;
        previous_y = y;
    }
}

// Placeholder implementations for the functions and types used in the code.
// These would need to be defined elsewhere in the actual application.
#[derive(Copy, Clone)]
enum Color {
    Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray,
    DarkGray, LightBlue, LightGreen, LightCyan, LightRed, LightMagenta,
    Yellow, White,
}

fn wait(_ms: u32) {
    // Placeholder
}

fn last_key() -> Option<char> {
    // Placeholder, should return the last pressed key if any
    None
}

fn mouse_x() -> i32 {
    // Placeholder, should return current mouse x position
    0
}

fn mouse_y() -> i32 {
    // Placeholder, should return current mouse y position
    0
}

fn left_mouse_pressed() -> bool {
    // Placeholder, should return if the left mouse button is pressed
    false
}

fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color) {
    // Placeholder
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Placeholder
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _width: u32, _color: Color) {
    // Placeholder
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;