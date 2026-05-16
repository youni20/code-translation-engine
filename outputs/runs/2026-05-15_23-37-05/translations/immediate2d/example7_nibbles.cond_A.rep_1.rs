use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;
use std::io;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
    lives: i32,
    score: i32,
    color: Color,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Color(u8, u8, u8);

// Global state
static mut SNAKES: [Snake; 2] = [
    Snake {
        body: VecDeque::new(),
        direction: Direction::Right,
        lives: 5,
        score: 0,
        color: Color(255, 255, 0), // Yellow
    },
    Snake {
        body: VecDeque::new(),
        direction: Direction::Left,
        lives: 5,
        score: 0,
        color: Color(255, 0, 255), // LightMagenta
    },
];

// Proxy methods to simulate pixel functionalities
fn draw_pixel(x: i32, y: i32, color: Color) {
    println!("Drawing pixel at ({}, {}) with color {:?}", x, y, color);
}

fn read_pixel(_x: i32, _y: i32) -> Color {
    Color(0, 0, 255) // Blue (Background)
}

fn wait(ms: u64) {
    sleep(Duration::from_millis(ms));
}

fn last_key() -> Option<char> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.chars().next()
}

fn clear_buffer() {
    // Clear input buffer
}

fn clear(color: Color) {
    println!("Clearing the screen to color {:?}", color);
}

fn draw_string(x: i32, y: i32, s: &str, color: Color, centered: bool) {
    println!("Drawing string '{}' at ({}, {}) with color {:?}", s, x, y, color);
}

fn set_big_pixel(x: i32, y: i32, color: Color) {
    draw_pixel(x * 2, y * 2 + 10, color);
    draw_pixel(x * 2 + 1, y * 2 + 10, color);
    draw_pixel(x * 2, y * 2 + 11, color);
    draw_pixel(x * 2 + 1, y * 2 + 11, color);
}

fn get_big_pixel(x: i32, y: i32) -> Color {
    read_pixel(x * 2, y * 2 + 10)
}

fn init_level(level: i32) {
    let background = Color(0, 0, 255); // Blue
    let walls = Color(255, 0, 0); // LightRed

    unsafe {
        clear(background);
        for s in SNAKES.iter_mut() {
            s.body.clear();
        }

        for i in 0..80 {
            set_big_pixel(i, 0, walls);
            set_big_pixel(i, 49, walls);
        }

        for j in 0..50 {
            set_big_pixel(0, j, walls);
            set_big_pixel(79, j, walls);
        }

        match level {
            1 => {
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Right;
                SNAKES[1].direction = Direction::Left;
            }
            2 => {
                for x in 20..60 {
                    set_big_pixel(x, 25, walls);
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            _ => {
                // Handle other levels
            }
        }
    }
}

fn advance_point(p: Point, d: Direction) -> Point {
    match d {
        Direction::Left => Point { x: p.x - 1, y: p.y },
        Direction::Right => Point { x: p.x + 1, y: p.y },
        Direction::Up => Point { x: p.x, y: p.y - 1 },
        Direction::Down => Point { x: p.x, y: p.y + 1 },
    }
}

fn read_input_and_delay() -> bool {
    wait(80);
    unsafe {
        let mut desired = [
            SNAKES[0].direction,
            SNAKES[1].direction
        ];

        if let Some(input) = last_key() {
            match input {
                'w' | 'W' if SNAKES[1].direction != Direction::Down => {
                    desired[1] = Direction::Up;
                }
                'a' | 'A' if SNAKES[1].direction != Direction::Right => {
                    desired[1] = Direction::Left;
                }
                's' | 'S' if SNAKES[1].direction != Direction::Up => {
                    desired[1] = Direction::Down;
                }
                'd' | 'D' if SNAKES[1].direction != Direction::Left => {
                    desired[1] = Direction::Right;
                }
                'p' | 'P' => {
                    pause("Paused! Press Space");
                }
                _ => {}
            }

            if desired[0] != SNAKES[0].direction || desired[1] != SNAKES[1].direction {
                SNAKES[0].direction = desired[0];
                SNAKES[1].direction = desired[1];
            }
        }
    }

    false
}

fn pause(message: &str) {
    last_key();

    draw_string(80, 113, message, Color(255, 255, 255), true);
    while last_key() != Some(' ') {
        wait(1);
    }
    draw_string(80, 113, message, Color(0, 0, 255), true);

    clear_buffer();
}

fn main() {
    run();
}

fn run() {
    let text_color = Color(255, 255, 255);
    let message_color1 = Color(255, 255, 0); // Yellow
    let message_color2 = Color(255, 0, 255); // LightMagenta

    draw_string(80, 10, "C + +    N i b b l e s", text_color, true);
    draw_string(80, 26, "Eat apples while avoiding walls & snakes.", Color(192, 192, 192), true); // LightGray
    draw_string(80, 32, "The more you eat, the longer you become.", Color(192, 192, 192), true);
    draw_string(80, 50, "Player 1: Arrow keys", message_color1, true);
    draw_string(80, 56, "Player 2: W, A, S, D", message_color2, true);
    draw_string(80, 62, "P to Pause", Color(192, 192, 192), true);
    draw_string(80, 90, "Press any key to continue", text_color, true);

    unsafe {
        SNAKES[0].color = message_color1;
        SNAKES[1].color = message_color2;

        loop {
            clear(Color(0, 0, 0));
            let players = how_many_players();

            SNAKES[0].lives = 5;
            SNAKES[1].lives = 5;
            SNAKES[0].score = 0;
            SNAKES[1].score = 0;

            if players == 1 {
                SNAKES[1].lives = 0;
            }

            let mut level = 1;
            let mut pickup_count = 0;
            const PICKUP_LIMIT: i32 = 9;

            while SNAKES[0].lives > 0 || SNAKES[1].lives > 0 {
                init_level(level);

                pause(&format!("Level {}, push space", level));

                generate_apple();

                let mut dead = false;
                while !dead && pickup_count < PICKUP_LIMIT {
                    if read_input_and_delay() {
                        break;
                    }

                    for s in &mut SNAKES {
                        if s.lives == 0 {
                            continue;
                        }

                        let new_head = advance_point(s.body.front().copied().unwrap(), s.direction);
                        
                        let hit = get_big_pixel(new_head.x, new_head.y);
                        
                        if hit == Color(255, 255, 255) { // Apple
                            pickup_count += 1;
                        } else if hit != Color(0, 0, 255) { // Background
                            s.lives -= 1;
                            dead = true;
                            break;
                        }

                        set_big_pixel(new_head.x, new_head.y, s.color);
                        s.body.push_front(new_head);

                        let old_tail = s.body.pop_back().unwrap();
                        set_big_pixel(old_tail.x, old_tail.y, Color(0, 0, 255)); // Background
                    }
                }

                if pickup_count >= PICKUP_LIMIT {
                    level += 1;
                    pickup_count = 0;
                }
            }

            if !game_over_play_again() {
                break;
            }
        }
    }
}

fn generate_apple() {
    // Stub logic to place an apple
}

fn game_over_play_again() -> bool {
    draw_string(80, 66, "Play Again? (Y/N)", Color(255, 255, 255), true);

    while let Some(c) = last_key() {
        if c == 'y' || c == 'Y' {
            return true;
        }
        if c == 'n' || c == 'N' {
            return false;
        }
    }

    wait(1);
    false
}

fn how_many_players() -> i32 {
    draw_string(80, 40, "How many players (1 or 2)?", Color(192, 192, 192), true);

    while let Some(c) = last_key() {
        if c == '1' {
            return 1;
        }
        if c == '2' {
            return 2;
        }
    }

    wait(1);
    1
}