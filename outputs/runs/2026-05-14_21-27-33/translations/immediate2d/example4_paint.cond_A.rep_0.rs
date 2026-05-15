const WIDTH: i32 = 800; // Assuming a default width, replace with the actual value from "immediate2d.h"
const HEIGHT: i32 = 600; // Assuming a default height, replace with the actual value

#[derive(Copy, Clone)]
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
    // Implementation of drawing a rectangle on the canvas
}

fn draw_pixel(_x: i32, _y: i32, _color: Color) {
    // Implementation of drawing a pixel on the canvas
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {
    // Implementation of drawing a line on the canvas
}

fn wait(milliseconds: u64) {
    std::thread::sleep(std::time::Duration::from_millis(milliseconds));
}

fn last_key() -> Option<char> {
    // Implementation to get the last key pressed, or None if no key was pressed
    None
}

fn mouse_x() -> i32 {
    // Implementation to get the current x-coordinate of the mouse
    0
}

fn mouse_y() -> i32 {
    // Implementation to get the current y-coordinate of the mouse
    0
}

fn left_mouse_pressed() -> bool {
    // Implementation to check if the left mouse button is pressed
    false
}

fn run() {
    let colors: [Color; 16] = [
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

    let palette_entry_width = WIDTH / 8;
    let palette_entry_height = 6;
    let canvas_y = palette_entry_height * 3 + 2;

    for i in 0..16 {
        let x = palette_entry_width * (i % 8);
        let mut y = 0;
        if i >= 8 {
            y += palette_entry_height;
        }
        draw_rectangle(x, y, palette_entry_width, palette_entry_height, colors[i as usize]);
    }

    let mut current_color = 15;
    let mut previous_x = -1;
    let mut previous_y = -1;

    loop {
        wait(1);
        if let Some(key) = last_key() {
            if key == ' ' {
                draw_rectangle(0, canvas_y, WIDTH, HEIGHT, Color::Black);
            }
        }

        let x = mouse_x();
        let y = mouse_y();

        let mouse_offscreen = x < 0 || y < 0;
        let mouse_on_canvas = !mouse_offscreen && y > canvas_y;

        if !mouse_on_canvas || !left_mouse_pressed() {
            previous_x = -1;
            previous_y = -1;
        }

        if !left_mouse_pressed() {
            continue;
        }

        if y < palette_entry_height * 2 {
            let row = if y > palette_entry_height { 1 } else { 0 };
            let col = x * 8 / WIDTH;
            current_color = row * 8 + col;
            draw_rectangle(0, palette_entry_height * 2 + 1, WIDTH, palette_entry_height, colors[current_color as usize]);
        }

        if !mouse_on_canvas {
            continue;
        }

        if previous_x == -1 && previous_y == -1 {
            draw_pixel(x, y, colors[current_color as usize]);
            previous_x = x;
            previous_y = y;
            continue;
        }

        draw_line(previous_x, previous_y, x, y, 1, colors[current_color as usize]);
        previous_x = x;
        previous_y = y;
    }
}

fn main() {
    run();
}