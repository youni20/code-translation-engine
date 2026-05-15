struct Button {
    x: i32,
    y: i32,
    color: Color,
}

fn draw_button(button: &Button) {
    let button_size = 20;
    draw_rectangle(button.x, button.y, button_size, button_size, &Color::DarkGray);
    draw_rectangle(button.x, button.y, button_size - 1, button_size - 1, &Color::White);
    draw_rectangle(button.x + 1, button.y + 1, button_size - 2, button_size - 2, &button.color);
}

fn test_button(button: &Button, x: i32, y: i32) -> bool {
    let button_size = 20;
    x >= button.x && x < button.x + button_size && y >= button.y && y < button.y + button_size
}

fn run() {
    let button1 = Button {
        x: 10,
        y: 90,
        color: Color::Blue,
    };

    let button2 = Button {
        x: 40,
        y: 90,
        color: Color::Green,
    };

    draw_button(&button1);
    draw_button(&button2);

    loop {
        wait(1);

        if !left_mouse_pressed() {
            continue;
        }

        let x = mouse_x();
        let y = mouse_y();

        if test_button(&button1, x, y) {
            draw_rectangle(0, 0, WIDTH, 80, &Color::Blue);
        }

        if test_button(&button2, x, y) {
            draw_rectangle(0, 0, WIDTH, 80, &Color::Green);
        }
    }
}

// Mock functions to replicate missing functionality
#[derive(Clone, Copy)]
enum Color {
    DarkGray,
    White,
    Blue,
    Green,
}

// Assume the following functions are provided by a library
fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: &Color) {}
fn wait(_ms: i32) {}
fn left_mouse_pressed() -> bool { false }
fn mouse_x() -> i32 { 0 }
fn mouse_y() -> i32 { 0 }

const WIDTH: i32 = 640;  // Example screen width

// Adding a main function to resolve the error
fn main() {
    run();
}