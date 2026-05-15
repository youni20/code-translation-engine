const BUTTON_SIZE: i32 = 20;
const WIDTH: i32 = 240; // Assuming a width for the screen for illustration purposes.

#[derive(Copy, Clone)]
struct Button {
    x: i32,
    y: i32,
    color: Color,
}

#[derive(Copy, Clone)]
enum Color {
    Blue,
    Green,
    DarkGray,
    White,
}

impl Color {
    fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Blue => (0, 0, 255),
            Color::Green => (0, 255, 0),
            Color::DarkGray => (169, 169, 169),
            Color::White => (255, 255, 255),
        }
    }
}

// Hypothetical functions to represent the graphics and input handling
fn draw_rectangle(x: i32, y: i32, width: i32, height: i32, color: Color) {
    let (r, g, b) = color.to_rgb();
    // Drawing logic with RGB values continues...
}

fn left_mouse_pressed() -> bool {
    // Hypothetically returns true if the left mouse button is pressed
    false
}

fn mouse_x() -> i32 {
    // Hypothetically returns the current mouse x coordinate
    0
}

fn mouse_y() -> i32 {
    // Hypothetically returns the current mouse y coordinate
    0
}

fn wait(ms: u64) {
    // Hypothetical wait function, for example:
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

fn draw_button(button: Button) {
    draw_rectangle(button.x, button.y, BUTTON_SIZE, BUTTON_SIZE, Color::DarkGray);
    draw_rectangle(button.x, button.y, BUTTON_SIZE - 1, BUTTON_SIZE - 1, Color::White);
    draw_rectangle(button.x + 1, button.y + 1, BUTTON_SIZE - 2, BUTTON_SIZE - 2, button.color);
}

fn test_button(button: Button, x: i32, y: i32) -> bool {
    x >= button.x && x < button.x + BUTTON_SIZE && y >= button.y && y < button.y + BUTTON_SIZE
}

fn run() {
    let button1 = Button { x: 10, y: 90, color: Color::Blue };
    let button2 = Button { x: 40, y: 90, color: Color::Green };

    draw_button(button1);
    draw_button(button2);

    // Loop forever
    loop {
        wait(1);

        if !left_mouse_pressed() {
            continue;
        }

        let x = mouse_x();
        let y = mouse_y();

        if test_button(button1, x, y) {
            draw_rectangle(0, 0, WIDTH, 80, Color::Blue);
        }

        if test_button(button2, x, y) {
            draw_rectangle(0, 0, WIDTH, 80, Color::Green);
        }
    }
}

fn main() {
    run();
}