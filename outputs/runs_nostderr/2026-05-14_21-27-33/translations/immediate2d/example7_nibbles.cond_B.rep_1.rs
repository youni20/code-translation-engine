use std::collections::VecDeque;
use std::iter::Iterator;

// Assuming there are placeholder implementations for these functions
// from "immediate2d.h" in C++
fn draw_pixel(_x: i32, _y: i32, _color: Color) {}
fn read_pixel(_x: i32, _y: i32) -> Color { Color::Background }
fn clear(_color: Color) {}
fn last_key() -> Option<char> { None }
fn last_buffered_key() -> Option<char> { None }
fn clear_input_buffer() {}
fn wait(_duration_ms: u32) {}
fn random_int(min: i32, _max: i32) -> i32 { min }
fn close_window() {}
fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _color: Color, _filled: bool) {}
fn play_music(_note: i32, _duration: u32) {}
fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq)]
enum Color {
    LightRed,
    Blue,
    White,
    Yellow,
    LightMagenta,
    DarkGray,
    Transparent,
    Black,
    Background,
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
        color: Color::Yellow,
    },
    Snake {
        body: VecDeque::new(),
        direction: Direction::Left,
        lives: 5,
        score: 0,
        color: Color::LightMagenta,
    },
];

fn set_big_pixel(x: i32, y: i32, color: Color) {
    draw_pixel(x * 2, y * 2 + 10, color);
    draw_pixel(x * 2 + 1, y * 2 + 10, color);
    draw_pixel(x * 2, y * 2 + 11, color);
    draw_pixel(x * 2 + 1, y * 2 + 11, color);
}

fn get_big_pixel(x: i32, y: i32) -> Color {
    read_pixel(x * 2, y * 2 + 10)
}

fn draw_string(x: i32, y: i32, s: &str, c: Color, centered: bool) {
    static FONT: [u32; 0] = [];

    let mut x_offset = x;
    if centered {
        let width: i32 = s
            .chars()
            .map(|ch| {
                if ch < ' ' || ch > '~' {
                    0
                } else {
                    ((FONT[ch as usize - 32] >> 28) + 1) as i32
                }
            })
            .sum();
        x_offset -= width / 2;
    }

    for i in s.chars() {
        if i < ' ' || i > '~' {
            continue;
        }
        let mut glyph = FONT[i as usize - 32];
        let width = glyph >> 28;
        for u in x_offset..x_offset + width as i32 {
            for v in y..y + 5 {
                if (glyph & 1) == 1 {
                    draw_pixel(u, v, c);
                }
                glyph >>= 1;
            }
        }
        if width > 0 {
            x_offset += width as i32 + 1;
        }
    }
}

fn init_level(level: i32) {
    unsafe { 
        clear(Color::Background);
        for s in SNAKES.iter_mut() {
            s.body.clear();
        }

        for i in 0..80 {
            set_big_pixel(i, 0, Color::LightRed);
            set_big_pixel(i, 49, Color::LightRed);
        }

        for j in 0..50 {
            set_big_pixel(0, j, Color::LightRed);
            set_big_pixel(79, j, Color::LightRed);
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
                    set_big_pixel(x, 25, Color::LightRed);
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            3 => {
                for y in 10..40 {
                    set_big_pixel(20, y, Color::LightRed);
                    set_big_pixel(60, y, Color::LightRed);
                }
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            4 => {
                for y in 0..30 {
                    set_big_pixel(20, y, Color::LightRed);
                    set_big_pixel(60, 49 - y, Color::LightRed);
                }
                for x in 0..40 {
                    set_big_pixel(x, 36, Color::LightRed);
                    set_big_pixel(81 - x, 13, Color::LightRed);
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            5 => {
                for y in 11..37 {
                    set_big_pixel(21, y, Color::LightRed);
                    set_big_pixel(58, y, Color::LightRed);
                }
                for x in 23..57 {
                    set_big_pixel(x, 9, Color::LightRed);
                    set_big_pixel(x, 38, Color::LightRed);
                }
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            6 => {
                for y in 0..49 {
                    if y >= 19 && y <= 30 {
                        continue;
                    }
                    for x in (10..=70).step_by(10) {
                        set_big_pixel(x, y, Color::LightRed);
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            7 => {
                for y in (1..49).step_by(2) {
                    set_big_pixel(40, y, Color::LightRed);
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            8 => {
                for y in 0..40 {
                    for x in (10..=70).step_by(10) {
                        set_big_pixel(x, if x % 20 == 0 { 49 - y } else { y }, Color::LightRed);
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            9 => {
                for i in 3..47 {
                    set_big_pixel(i + 2, i, Color::LightRed);
                    set_big_pixel(i + 28, i, Color::LightRed);
                }
                SNAKES[0].body.push_front(Point { x: 75, y: 40 });
                SNAKES[1].body.push_front(Point { x: 5, y: 15 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            _ => {
                for y in (1..49).step_by(2) {
                    for j in 0..7 {
                        set_big_pixel(j * 10 + 10, y + if j % 2 == 0 { 0 } else { 1 }, Color::LightRed);
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }

        for s in SNAKES.iter_mut() {
            s.body.push_back(s.body.back().unwrap().clone());
        }
    }
}

fn erase_snakes() {
    unsafe {
        for _ in 0..8 {
            for s in SNAKES.iter_mut() {
                for j in (0..s.body.len()).step_by(8) {
                    let segment = &s.body[j];
                    set_big_pixel(segment.x, segment.y, Color::Background);
            }
        }
            wait(35);
        }
    }
}

fn pause(message: &str) {
    last_key();

    draw_string(80, 113, message, Color::White, true);
    while last_key() != Some(' ') {
        wait(1);
    }
    draw_string(80, 113, message, Color::Background, true);

    clear_input_buffer();
}

fn generate_apple() {
    loop {
        let x = random_int(1, 79);
        let y = random_int(1, 49);
        if get_big_pixel(x, y) == Color::Background {
            set_big_pixel(x, y, Color::White);
            return;
        }
    }
}

fn read_input_and_delay() -> bool {
    wait(80);

    unsafe {
        let mut desired = [SNAKES[0].direction, SNAKES[1].direction];
        
        while let Some(k) = last_buffered_key() {
            match k {
                'p' | 'P' => pause("Paused! Press Space"),
                'w' | 'W' if SNAKES[1].direction != Direction::Down => desired[1] = Direction::Up,
                'a' | 'A' if SNAKES[1].direction != Direction::Right => desired[1] = Direction::Left,
                's' | 'S' if SNAKES[1].direction != Direction::Up => desired[1] = Direction::Down,
                'd' | 'D' if SNAKES[1].direction != Direction::Left => desired[1] = Direction::Right,
                '↑' if SNAKES[0].direction != Direction::Down => desired[0] = Direction::Up,
                '←' if SNAKES[0].direction != Direction::Right => desired[0] = Direction::Left,
                '↓' if SNAKES[0].direction != Direction::Up => desired[0] = Direction::Down,
                '→' if SNAKES[0].direction != Direction::Left => desired[0] = Direction::Right,
                '\x1B' => {
                    close_window();
                    return true;
                }
                _ => {},
            }

            if desired[0] != SNAKES[0].direction || desired[1] != SNAKES[1].direction {
                break;
            }
        }

        SNAKES[0].direction = desired[0];
        SNAKES[1].direction = desired[1];
        false
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

fn draw_scores(player_count: i32) {
    draw_rectangle(0, 0, 160, 10, Color::Background, true);

    unsafe {
        if player_count > 0 {
            draw_string(5, 3, &format!("{} <-- Sammy", SNAKES[0].score), SNAKES[0].color, true);
        }
        if player_count > 1 {
            draw_string(110, 3, &format!("Jake --> {}", SNAKES[1].score), SNAKES[1].color, true);
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

    draw_rectangle(41, 41, 80, 40, Color::Black, false);
    draw_rectangle(40, 40, 80, 40, Color::DarkGray, true);
    draw_rectangle(40, 40, 80, 40, Color::Transparent, true);
    draw_string(80, 50, "G A M E   O V E R", Color::White, true);
    draw_string(80, 66, "Play Again? (Y/N)", Color::White, true);

    loop {
        if let Some(c) = last_key() {
            match c {
                'y' | 'Y' => return true,
                'n' | 'N' => return false,
                _ => {},
            }
        }
        wait(1);
    }
}

fn how_many_players() -> i32 {
    last_key();

    draw_string(80, 40, "How many players (1 or 2)?", Color::White, true);

    loop {
        if let Some(c) = last_key() {
            match c {
                '1' => return 1,
                '2' => return 2,
                _ => {},
            }
        }
        wait(1);
    }
}

const MS_PER_MINUTE: u32 = 60000;
const BPM: u32 = 160;
const BEATS_PER_MS: u32 = MS_PER_MINUTE / BPM;

const DURATION_4: u32 = BEATS_PER_MS * 1 / 1;
const DURATION_8: u32 = BEATS_PER_MS * 1 / 2;
const DURATION_10: u32 = BEATS_PER_MS * 2 / 5;
const DURATION_16: u32 = BEATS_PER_MS * 1 / 4;
const DURATION_20: u32 = BEATS_PER_MS * 1 / 5;
const DURATION_32: u32 = BEATS_PER_MS * 1 / 8;

fn play_title_music() {
    for &n in &[0, 48, 50, 52, 50, 48, 50] {
        play_music(n, DURATION_8);
    }
    for &n in &[52, 48, 48] {
        play_music(n, DURATION_4);
    }
}

fn play_level_start() {
    for &n in &[60, 62, 64, 62, 60, 62] {
        play_music(n, DURATION_20);
    }
    for &n in &[64, 60, 60] {
        play_music(n, DURATION_10);
    }
}

fn play_apple_pickup() {
    for &n in &[48, 48, 48, 52] {
        play_music(n, DURATION_16);
    }
}

fn play_snake_dead() {
    for &n in &[36, 37, 39, 36, 37, 34, 32] {
        play_music(n, DURATION_32);
    }
}

fn run() {
    draw_string(80, 10, "C + +    N i b b l e s", Color::White, true);
    draw_line(47, 17, 111, 17, 1, Color::DarkGray);
    draw_string(80, 26, "Eat apples while avoiding walls & snakes.", Color::White, true);
    draw_string(80, 32, "The more you eat, the longer you become.", Color::White, true);
    draw_string(80, 50, "Player 1: Arrow keys", Color::Yellow, true);
    draw_string(80, 56, "Player 2: W, A, S, D", Color::LightMagenta, true);
    draw_string(80, 62, "P to Pause", Color::White, true);
    draw_string(80, 90, "Press any key to continue", Color::White, true);

    play_title_music();
    while last_key().is_none() {
        wait(1);
    }

    unsafe {
        SNAKES[0].color = Color::Yellow;
        SNAKES[1].color = Color::LightMagenta;
    }

    loop {
        clear(Color::Background);
        let players = how_many_players();

        unsafe {
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
                draw_scores(players);
                pause(&format!("Level {}, push space", level));
                play_level_start();

                generate_apple();

                let mut dead = false;
                while !dead && pickup_count < PICKUP_LIMIT {
                    if read_input_and_delay() {
                        break;
                    }

                    for s in SNAKES.iter_mut() {
                        if s.lives == 0 {
                            continue;
                        }

                        let new_head = advance_point(s.body.front().unwrap().clone(), s.direction);

                        let hit = get_big_pixel(new_head.x, new_head.y);
                        if hit == Color::White {
                            play_apple_pickup();
                            pickup_count += 1;
                            if pickup_count < PICKUP_LIMIT {
                                generate_apple();
                            }

                            s.score += pickup_count;
                            let new_segments = pickup_count * 4;
                            for _ in 0..new_segments {
                                s.body.push_back(s.body.back().unwrap().clone());
                            }

                            draw_scores(players);
                        } else if hit != Color::Background {
                            play_snake_dead();

                            s.lives -= 1;
                            dead = true;
                            break;
                        }

                        set_big_pixel(new_head.x, new_head.y, s.color);
                        s.body.push_front(new_head);

                        let old_tail = s.body.pop_back().unwrap();
                        set_big_pixel(old_tail.x, old_tail.y, Color::Background);
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
            }
        }
    }
}

fn main() {
    run();
}