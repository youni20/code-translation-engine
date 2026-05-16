struct Button {
    x: i32,
    y: i32,
    color: Color,
}

#[derive(Clone)]
enum Color {
    DarkGray,
    White,
    Blue,
    Green,
}

const BUTTON_SIZE: i32 = 20;

// Placeholder functions to mimic the C++ environment
fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color) {
    // Implement drawing logic
}

fn left_mouse_pressed() -> bool {
    // Implement mouse press detection
    false
}

fn mouse_x() -> i32 {
    // Implement mouse x-coordinate retrieval
    0
}

fn mouse_y() -> i32 {
    // Implement mouse y-coordinate retrieval
    0
}

fn wait(_ms: u32) {
    // Implement waiting logic
}

fn draw_button(button: &Button) {
    draw_rectangle(button.x, button.y, BUTTON_SIZE, BUTTON_SIZE, Color::DarkGray);
    draw_rectangle(button.x, button.y, BUTTON_SIZE - 1, BUTTON_SIZE - 1, Color::White);
    draw_rectangle(button.x + 1, button.y + 1, BUTTON_SIZE - 2, BUTTON_SIZE - 2, button.color.clone());
}

fn test_button(button: &Button, x: i32, y: i32) -> bool {
    x >= button.x && x < button.x + BUTTON_SIZE && y >= button.y && y < button.y + BUTTON_SIZE
}

fn run() {
    let button1 = Button { x: 10, y: 90, color: Color::Blue };
    let button2 = Button { x: 40, y: 90, color: Color::Green };

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
            draw_rectangle(0, 0, 800, 80, Color::Blue); // Assuming Width is 800
        }

        if test_button(&button2, x, y) {
            draw_rectangle(0, 0, 800, 80, Color::Green); // Assuming Width is 800
        }
    }
}

fn main() {
    run();
}