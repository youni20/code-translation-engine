#![allow(non_snake_case)]

#[derive(Clone, Copy)]
struct Color;

// Placeholder for Color constants
const Black: Color = Color;
const Blue: Color = Color;
const Green: Color = Color;
const Cyan: Color = Color;
const Red: Color = Color;
const Magenta: Color = Color;
const Brown: Color = Color;
const LightGray: Color = Color;
const DarkGray: Color = Color;
const LightBlue: Color = Color;
const LightGreen: Color = Color;
const LightCyan: Color = Color;
const LightRed: Color = Color;
const LightMagenta: Color = Color;
const Yellow: Color = Color;
const White: Color = Color;

// Placeholder for Width and Height constants/functions
const Width: i32 = 800;
const Height: i32 = 600;

// Placeholder functions (to be replaced with actual implementation)
fn DrawRectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color) {}
fn DrawPixel(_x: i32, _y: i32, _color: Color) {}
fn DrawLine(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {}
fn Wait(_ms: u64) {}
fn LastKey() -> Option<char> { None }
fn MouseX() -> i32 { 0 }
fn MouseY() -> i32 { 0 }
fn LeftMousePressed() -> bool { false }

fn run() {
    let colors: [Color; 16] = [
        Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray, DarkGray, 
        LightBlue, LightGreen, LightCyan, LightRed, LightMagenta, Yellow, White,
    ];

    let PaletteEntryWidth = Width / 8;
    let PaletteEntryHeight = 6;
    let CanvasY = PaletteEntryHeight * 3 + 2;

    for i in 0..16 {
        let x = PaletteEntryWidth * (i % 8);
        let mut y = 0;

        if i >= 8 {
            y += PaletteEntryHeight;
        }

        DrawRectangle(x, y, PaletteEntryWidth, PaletteEntryHeight, colors[i as usize]);
    }

    let mut currentColor = 15;
    let mut previousX = -1;
    let mut previousY = -1;

    loop {
        Wait(1);

        if let Some(key) = LastKey() {
            if key == ' ' {
                DrawRectangle(0, CanvasY, Width, Height, Black);
            }
        }

        let x = MouseX();
        let y = MouseY();

        let mouseOffscreen = x < 0 || y < 0;
        let mouseOnCanvas = !mouseOffscreen && y > CanvasY;

        if !mouseOnCanvas || !LeftMousePressed() {
            previousX = -1;
            previousY = -1;
        }

        if !LeftMousePressed() {
            continue;
        }

        if y < PaletteEntryHeight * 2 {
            let row = if y > PaletteEntryHeight { 1 } else { 0 };
            let col = x * 8 / Width;
            currentColor = row * 8 + col;
            DrawRectangle(0, PaletteEntryHeight * 2 + 1, Width, PaletteEntryHeight, colors[currentColor as usize]);
        }

        if !mouseOnCanvas {
            continue;
        }

        if previousX == -1 && previousY == -1 {
            DrawPixel(x, y, colors[currentColor as usize]);
            previousX = x;
            previousY = y;
            continue;
        }

        DrawLine(previousX, previousY, x, y, 1, colors[currentColor as usize]);
        previousX = x;
        previousY = y;
    }
}

fn main() {
    run();
}