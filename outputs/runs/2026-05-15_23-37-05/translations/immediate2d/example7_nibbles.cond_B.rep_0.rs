use std::collections::VecDeque;

// Example 7 - Nibbles
// A snake game modeled after the original NIBBLES.BAS included with Microsoft QBasic

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
#[derive(PartialEq)]
struct Color(u32);

#[derive(Clone, Copy)]
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

static mut SNAKES: [Snake; 2] = [
    Snake {
        body: VecDeque::new(),
        direction: Direction::Right,
        lives: 5,
        score: 0,
        color: Color(0), // Example color value
    },
    Snake {
        body: VecDeque::new(),
        direction: Direction::Left,
        lives: 5,
        score: 0,
        color: Color(0), // Example color value
    },
];

// Utility functions to interface with a hypothetical graphics library
fn draw_pixel(_: i32, _: i32, _: Color) {}
fn read_pixel(_: i32, _: i32) -> Color {
    Color(0) // Example return value
}
fn clear(_: Color) {}
fn draw_rectangle(_: i32, _: i32, _: i32, _: i32, _: Color, _: bool) {}
fn draw_line(_: i32, _: i32, _: i32, _: i32, _: i32, _: Color) {}
fn random_int(_: i32, _: i32) -> i32 {
    0 // Example return value
}
fn play_music(_: i32, _: i32) {}
fn wait(_: i32) {}
fn last_key() -> Option<char> {
    None // Example return value
}
fn last_buffered_key() -> Option<char> {
    None // Example return value
}
fn clear_input_buffer() {}
const ESC: char = 27 as char;
const UP: char = 'W';
const LEFT: char = 'A';
const DOWN: char = 'S';
const RIGHT: char = 'D';
fn close_window() {}

fn set_big_pixel(x: i32, y: i32, c: Color) {
    draw_pixel(x * 2, y * 2 + 10, c);
    draw_pixel(x * 2 + 1, y * 2 + 10, c);
    draw_pixel(x * 2, y * 2 + 11, c);
    draw_pixel(x * 2 + 1, y * 2 + 11, c);
}

fn get_big_pixel(x: i32, y: i32) -> Color {
    read_pixel(x * 2, y * 2 + 10)
}

fn draw_string(x: i32, y: i32, s: &str, c: Color, centered: bool) {
    // Simplified placeholder implementation
    if centered {
        let width: i32 = s.chars().map(|_| 8).sum();
        let x = x - width / 2;
        for (i, _) in s.chars().enumerate() {
            draw_pixel(x + i as i32 * 8, y, c); // Simplified character rendering
        }
    } else {
        for (i, _) in s.chars().enumerate() {
            draw_pixel(x + i as i32 * 8, y, c); // Simplified character rendering
        }
    }
}

fn init_level(level: i32) {
    unsafe {
        clear(Color(0x0000FF)); // Replace Blue with hex value
        for s in &mut SNAKES {
            s.body.clear();
        }

        // Draw perimeter walls
        for i in 0..80 {
            set_big_pixel(i, 0, Color(0xFF0000)); // Replace LightRed with hex value
            set_big_pixel(i, 49, Color(0xFF0000));
        }
        for j in 0..50 {
            set_big_pixel(0, j, Color(0xFF0000));
            set_big_pixel(79, j, Color(0xFF0000));
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
                    set_big_pixel(x, 25, Color(0xFF0000));
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            3 => {
                for y in 10..40 {
                    set_big_pixel(20, y, Color(0xFF0000));
                    set_big_pixel(60, y, Color(0xFF0000));
                }
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            4 => {
                for y in 0..30 {
                    set_big_pixel(20, y, Color(0xFF0000));
                    set_big_pixel(60, 49 - y, Color(0xFF0000));
                }
                for x in 0..40 {
                    set_big_pixel(x, 36, Color(0xFF0000));
                    set_big_pixel(81 - x, 13, Color(0xFF0000));
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            5 => {
                for y in 11..37 {
                    set_big_pixel(21, y, Color(0xFF0000));
                    set_big_pixel(58, y, Color(0xFF0000));
                }
                for x in 23..57 {
                    set_big_pixel(x, 9, Color(0xFF0000));
                    set_big_pixel(x, 38, Color(0xFF0000));
                }
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            6 => {
                for y in 0..49 {
                    if !(19..=30).contains(&y) {
                        for x in (10..=70).step_by(10) {
                            set_big_pixel(x, y, Color(0xFF0000));
                        }
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            7 => {
                for y in (1..49).step_by(2) {
                    set_big_pixel(40, y, Color(0xFF0000));
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            8 => {
                for y in 0..40 {
                    for x in (10..=70).step_by(10) {
                        set_big_pixel(x, if x % 20 == 0 { 49 - y } else { y }, Color(0xFF0000));
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            9 => {
                for i in 3..47 {
                    set_big_pixel(i + 2, i, Color(0xFF0000));
                    set_big_pixel(i + 28, i, Color(0xFF0000));
                }
                SNAKES[0].body.push_front(Point { x: 75, y: 40 });
                SNAKES[1].body.push_front(Point { x: 5, y: 15 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            _ => {
                for y in (1..49).step_by(2) {
                    for j in 0..7 {
                        set_big_pixel(j * 10 + 10, y + if j % 2 == 0 { 0 } else { 1 }, Color(0xFF0000));
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }

        for s in &mut SNAKES {
            s.body.push_back(s.body[0]);
        }
    }
}

fn erase_snakes() {
    unsafe {
        for i in 0..8 {
            for s in &mut SNAKES {
                for j in (i..s.body.len()).step_by(8) {
                    let point = s.body[j];
                    set_big_pixel(point.x, point.y, Color(0x0000FF)); // Replace with Background color
                }
            }
            wait(35);
        }
    }
}

fn pause(message: &str) {
    last_key();

    draw_string(80, 113, message, Color(0xFFFFFF), true); // Replace White with hex value
    while last_key() != Some(' ') {
        wait(1);
    }
    draw_string(80, 113, message, Color(0x0000FF), true); // Replace Background with hex value

    clear_input_buffer();
}

fn generate_apple() {
    loop {
        let x = random_int(1, 79);
        let y = random_int(1, 49);
        if get_big_pixel(x, y) != Color(0x0000FF) {
            continue;
        }

        set_big_pixel(x, y, Color(0xFFFFFF));
        return;
    }
}

fn read_input_and_delay() -> bool {
    wait(80);
    unsafe {
        let mut desired = [SNAKES[0].direction, SNAKES[1].direction];
        while let Some(k) = last_buffered_key() {
            match k {
                'p' | 'P' => pause("Paused! Press Space"),
                'w' | 'W' => {
                    if SNAKES[1].direction != Direction::Down {
                        desired[1] = Direction::Up;
                    }
                }
                'a' | 'A' => {
                    if SNAKES[1].direction != Direction::Right {
                        desired[1] = Direction::Left;
                    }
                }
                's' | 'S' => {
                    if SNAKES[1].direction != Direction::Up {
                        desired[1] = Direction::Down;
                    }
                }
                'd' | 'D' => {
                    if SNAKES[1].direction != Direction::Left {
                        desired[1] = Direction::Right;
                    }
                }
                'W' => {
                    if SNAKES[0].direction != Direction::Down {
                        desired[0] = Direction::Up;
                    }
                }
                'A' => {
                    if SNAKES[0].direction != Direction::Right {
                        desired[0] = Direction::Left;
                    }
                }
                'S' => {
                    if SNAKES[0].direction != Direction::Up {
                        desired[0] = Direction::Down;
                    }
                }
                'D' => {
                    if SNAKES[0].direction != Direction::Left {
                        desired[0] = Direction::Right;
                    }
                }
                ESC => {
                    close_window();
                    return true;
                }
                _ => {}
            }

            if desired[0] != SNAKES[0].direction || desired[1] != SNAKES[1].direction {
                break;
            }
        }

        SNAKES[0].direction = desired[0];
        SNAKES[1].direction = desired[1];
    }
    false
}

fn advance_point(p: Point, d: Direction) -> Point {
    match d {
        Direction::Left => Point { x: p.x - 1, y: p.y },
        Direction::Right => Point { x: p.x + 1, y: p.y },
        Direction::Up => Point { x: p.x, y: p.y - 1 },
        Direction::Down => Point { x: p.x, y: p.y + 1 },
    }
}

fn draw_scores(player_count: i32) {
    const WIDTH: i32 = 160; // Example screen width
    draw_rectangle(0, 0, WIDTH, 10, Color(0x0000FF), true);

    unsafe {
        if player_count > 0 {
            draw_string(
                5,
                3,
                &format!("{} <-- Sammy", SNAKES[0].score),
                SNAKES[0].color,
                false,
            );
        }

        if player_count > 1 {
            draw_string(
                110,
                3,
                &format!("Jake --> {}", SNAKES[1].score),
                SNAKES[1].color,
                false,
            );
        }

        for i in 0..SNAKES[0].lives - 1 {
            draw_rectangle(2 + i * 6, 114, 4, 2, SNAKES[0].color, true);
        }

        for i in 0..SNAKES[1].lives - 1 {
            draw_rectangle(154 - i * 6, 114, 4, 2, SNAKES[1].color, true);
        }
    }
}

fn game_over_play_again() -> bool {
    last_key();

    draw_rectangle(41, 41, 80, 40, Color(0x000000), true); // Replace Black with hex value
    draw_rectangle(40, 40, 80, 40, Color(0x444444), true); // Replace DarkGray with hex value
    draw_rectangle(40, 40, 80, 40, Color(0x000000), false); // Replace Transparent with hex light gray
    draw_string(80, 50, "G A M E   O V E R", Color(0xCCCCCC), true); // Replace LightGray
    draw_string(80, 66, "Play Again? (Y/N)", Color(0xFFFFFF), true); // Replace White

    loop {
        let c = last_key();
        if c == Some('y') || c == Some('Y') {
            return true;
        }
        if c == Some('n') || c == Some('N') {
            return false;
        }
        wait(1);
    }
}

fn how_many_players() -> i32 {
    last_key();

    draw_string(80, 40, "How many players (1 or 2)?", Color(0xCCCCCC), true); // Replace LightGray

    loop {
        if let Some(c) = last_key() {
            if c == '1' {
                return 1;
            }
            if c == '2' {
                return 2;
            }
        }

        wait(1);
    }
}

fn play_title_music() {
    for n in [0, 48, 50, 52, 50, 48, 50].iter() {
        play_music(*n, 75);
    }
    for n in [52, 48, 48].iter() {
        play_music(*n, 150);
    }
}

fn play_level_start() {
    for n in [60, 62, 64, 62, 60, 62].iter() {
        play_music(*n, 37);
    }
    for n in [64, 60, 60].iter() {
        play_music(*n, 60);
    }
}

fn play_apple_pickup() {
    for n in [48, 48, 48, 52].iter() {
        play_music(*n, 18);
    }
}

fn play_snake_dead() {
    for n in [36, 37, 39, 36, 37, 34, 32].iter() {
        play_music(*n, 9);
    }
}

fn run() {
    draw_string(80, 10, "R u s t    N i b b l e s", Color(0xFFFFFF), true); // Replace White
    draw_line(47, 17, 111, 17, 1, Color(0x444444)); // Replace DarkGray
    draw_string(
        80,
        26,
        "Eat apples while avoiding walls & snakes.",
        Color(0xCCCCCC),
        true,
    ); // Replace LightGray
    draw_string(
        80,
        32,
        "The more you eat, the longer you become.",
        Color(0xCCCCCC),
        true,
    ); // Replace LightGray
    draw_string(80, 50, "Player 1: Arrow keys", Color(0xFFFF00), true); // Replace Yellow
    draw_string(80, 56, "Player 2: W, A, S, D", Color(0xFF00FF), true); // Replace LightMagenta
    draw_string(80, 62, "P to Pause", Color(0xCCCCCC), true); // Replace LightGray
    draw_string(80, 90, "Press any key to continue", Color(0xFFFFFF), true); // Replace White

    play_title_music();
    while last_key() == None {
        wait(1);
    }

    unsafe {
        SNAKES[0].color = Color(0xFFFF00); // Replace Yellow
        SNAKES[1].color = Color(0xFF00FF); // Replace LightMagenta
    }

    loop {
        clear(Color(0x000000)); // Clear screen with a color, replace with Background

        let players = how_many_players();

        unsafe {
            SNAKES[0].lives = 5;
            SNAKES[0].score = 0;

            SNAKES[1].lives = if players == 1 { 0 } else { 5 };
            SNAKES[1].score = 0;
        }

        let mut level = 1;
        let mut pickup_count = 0;
        const PICKUP_LIMIT: i32 = 9;

        loop {
            unsafe {
                if SNAKES[0].lives <= 0 && SNAKES[1].lives <= 0 {
                    break;
                }
            }

            init_level(level);
            draw_scores(players);
            pause(&format!("Level {}, push space", level));
            play_level_start();
            generate_apple();

            let mut dead = false;
            while !dead && pickup_count < PICKUP_LIMIT {
                if read_input_and_delay() {
                    break;
                }

                unsafe {
                    for s in &mut SNAKES {
                        if s.lives == 0 {
                            continue;
                        }

                        let new_head = advance_point(s.body.front().copied().unwrap(), s.direction);

                        let hit = get_big_pixel(new_head.x, new_head.y);
                        if hit == Color(0xFFFFFF) { // Replace Apple with its hex color
                            play_apple_pickup();
                            pickup_count += 1;
                            if pickup_count < PICKUP_LIMIT {
                                generate_apple();
                            }

                            s.score += pickup_count;
                            let new_segments = pickup_count * 4;
                            for _ in 0..new_segments {
                                s.body.push_back(*s.body.back().unwrap());
                            }

                            draw_scores(players);
                        } else if hit != Color(0x0000FF) { // Replace Background with its hex color
                            play_snake_dead();

                            s.lives -= 1;
                            dead = true;
                            break;
                        }

                        set_big_pixel(new_head.x, new_head.y, s.color);
                        s.body.push_front(new_head);

                        let old_tail = s.body.pop_back().unwrap();
                        set_big_pixel(old_tail.x, old_tail.y, Color(0x0000FF)); // Replace Background
                    }
                }
            }

            if pickup_count >= PICKUP_LIMIT {
                level += 1;
                pickup_count = 0;
            }

            erase_snakes();
        }

        if !game_over_play_again() {
            close_window();
            break;
        }
    }
}

fn main() {
    run();
}