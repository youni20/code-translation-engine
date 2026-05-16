struct Button {
    x: i32,
    y: i32,
    color: Color,
}

const BUTTON_SIZE: i32 = 20;

fn draw_rectangle(x: i32, y: i32, width: i32, height: i32, color: Color) {
    // Implementation of drawing a rectangle goes here
}

fn wait(milliseconds: u32) {
    // Implementation of wait functionality goes here
}

fn left_mouse_pressed() -> bool {
    // Implementation to check if left mouse is pressed goes here
    true // Placeholder return value
}

fn mouse_x() -> i32 {
    // Implementation to get mouse x position goes here
    0 // Placeholder return value
}

fn mouse_y() -> i32 {
    // Implementation to get mouse y position goes here
    0 // Placeholder return value
}

fn draw_button(button: &Button) {
    draw_rectangle(button.x, button.y, BUTTON_SIZE, BUTTON_SIZE, Color::DarkGray);
    draw_rectangle(button.x, button.y, BUTTON_SIZE - 1, BUTTON_SIZE - 1, Color::White);
    draw_rectangle(button.x + 1, button.y + 1, BUTTON_SIZE - 2, BUTTON_SIZE - 2, button.color);
}

fn test_button(button: &Button, x: i32, y: i32) -> bool {
    x >= button.x && x < button.x + BUTTON_SIZE &&
    y >= button.y && y < button.y + BUTTON_SIZE
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
            draw_rectangle(0, 0, WIDTH, 80, Color::Blue);
        }

        if test_button(&button2, x, y) {
            draw_rectangle(0, 0, WIDTH, 80, Color::Green);
        }
    }
}

fn main() {
    run();
}

#[derive(Copy, Clone)]
enum Color {
    DarkGray,
    White,
    Blue,
    Green,
}

const WIDTH: i32 = 100;