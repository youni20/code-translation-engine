// Since immediate2d is not a real crate, we will mock the definitions to ensure this code compiles.
// These mocks are for compilation purposes only and do not represent actual functionality.

struct Color;
struct Black;
struct Blue;
struct Green;
struct Cyan;
struct Red;
struct Magenta;
struct Brown;
struct LightGray;
struct DarkGray;
struct LightBlue;
struct LightGreen;
struct LightCyan;
struct LightRed;
struct LightMagenta;
struct Yellow;
struct White;

fn Width() -> i32 { 640 }
fn Height() -> i32 { 480 }
fn LastKey() -> char { ' ' }
fn MouseX() -> i32 { 0 }
fn MouseY() -> i32 { 0 }
fn LeftMousePressed() -> bool { false }
fn Wait(_: i32) {}
fn DrawLine(_: i32, _: i32, _: i32, _: i32, _: i32, _: Color) {}
fn DrawPixel(_: i32, _: i32, _: Color) {}
fn DrawRectangle(_: i32, _: i32, _: i32, _: i32, _: Color) {}

fn main() {
    run();
}

fn run() {
    let colors: [Color; 16] = [
        Black as Color, Blue as Color, Green as Color, Cyan as Color, Red as Color, Magenta as Color, 
        Brown as Color, LightGray as Color, DarkGray as Color, LightBlue as Color, 
        LightGreen as Color, LightCyan as Color, LightRed as Color, LightMagenta as Color, 
        Yellow as Color, White as Color,
    ];

    let palette_entry_width = Width() / 8;
    let palette_entry_height = 6;
    let canvas_y = palette_entry_height * 3 + 2;

    for i in 0..16 {
        let x = palette_entry_width * (i % 8);
        let mut y = 0;

        if i >= 8 {
            y += palette_entry_height;
        }

        DrawRectangle(x, y, palette_entry_width, palette_entry_height, colors[i]);
    }

    let mut current_color = 15;
    let mut previous_x = -1;
    let mut previous_y = -1;

    loop {
        Wait(1);

        let key = LastKey();

        if key == ' ' {
            DrawRectangle(0, canvas_y, Width(), Height(), Black as Color);
        }

        let x = MouseX();
        let y = MouseY();

        let mouse_offscreen = x < 0 || y < 0;
        let mouse_on_canvas = !mouse_offscreen && y > canvas_y;

        if !mouse_on_canvas || !LeftMousePressed() {
            previous_x = -1;
            previous_y = -1;
        }

        if !LeftMousePressed() {
            continue;
        }

        if y < palette_entry_height * 2 {
            let row = if y > palette_entry_height { 1 } else { 0 };
            let col = x * 8 / Width();

            current_color = row * 8 + col;

            DrawRectangle(0, palette_entry_height * 2 + 1, Width(), palette_entry_height, colors[current_color]);
        }

        if !mouse_on_canvas {
            continue;
        }

        if previous_x == -1 && previous_y == -1 {
            DrawPixel(x, y, colors[current_color]);
            previous_x = x;
            previous_y = y;
            continue;
        }

        DrawLine(previous_x, previous_y, x, y, 1, colors[current_color]);

        previous_x = x;
        previous_y = y;
    }
}