use std::collections::VecDeque;
use std::cmp::max;

// Simulated external dependencies
#[derive(Clone, Copy, PartialEq)]
enum Color {
    LightRed,
    Blue,
    White,
    Yellow,
    LightMagenta,
    DarkGray,
    Black,
    Transparent,
    LightGray,
}

fn DrawPixel(_x: i32, _y: i32, _c: Color) {}
fn ReadPixel(_x: i32, _y: i32) -> Color { Color::Blue }
fn Clear(_color: Color) {}
fn Wait(_ms: u32) {}
fn LastKey() -> Option<char> { None }
fn LastBufferedKey() -> Option<char> { None }
fn ClearInputBuffer() {}
fn CloseWindow() {}
fn DrawRectangle(_x: i32, _y: i32, _w: i32, _h: i32, _c: Color, _fill: bool) {}
fn DrawLine(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _c: Color) {}
fn RandomInt(_min: i32, _max: i32) -> i32 { 0 }
fn PlayMusic(_note: i32, _duration: i32) {}

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

struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
    lives: i32,
    score: i32,
    color: Color,
}

static mut SNAKES: [Snake; 2] = [
    Snake { body: VecDeque::new(), direction: Direction::Right, lives: 5, score: 0, color: Color::Yellow },
    Snake { body: VecDeque::new(), direction: Direction::Left, lives: 5, score: 0, color: Color::LightMagenta },
];

fn set_big_pixel(x: i32, y: i32, c: Color) {
    DrawPixel(x * 2, y * 2 + 10, c);
    DrawPixel(x * 2 + 1, y * 2 + 10, c);
    DrawPixel(x * 2, y * 2 + 11, c);
    DrawPixel(x * 2 + 1, y * 2 + 11, c);
}

fn get_big_pixel(x: i32, y: i32) -> Color {
    ReadPixel(x * 2, y * 2 + 10)
}

fn draw_string(x: i32, y: i32, s: &str, c: Color, centered: bool) {
    static FONT: [u32; 96] = [
        0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA, 0x10000003,
        0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 0x30000C98,
        0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21,
        0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 0x30001151, 0x30000AA1,
        0x506ADE2E, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E,
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x2000023F, 0x30006083, 0x200003F1, 0x30000822, 0x30004210,
        0x20000041, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E,
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
    ];

    let mut draw_x = x;

    if centered {
        draw_x -= s.chars().filter_map(|c| if (32..128).contains(&(c as u8)) {
            Some((FONT[c as usize - 32] >> 28) as i32 + 1)
        } else { 
            None
        }).sum::<i32>() / 2;
    }

    for i in s.chars() {
        if !(32..128).contains(&(i as u8)) { continue; }
        let glyph = FONT[i as usize - 32];
        let width = (glyph >> 28) as i32;
        if width > 0 {
            for u in draw_x..draw_x + width {
                for v in y..y + 5 {
                    if (glyph & (1 << ((u - draw_x) + (v - y) * width) as u32)) != 0 {
                        DrawPixel(u, v, c);
                    }
                }
            }
            draw_x += width + 1;
        }
    }
}

const WALLS: Color = Color::LightRed;
const BACKGROUND: Color = Color::Blue;
const TEXT: Color = Color::White;
const APPLE: Color = Color::White;
const PICKUP_LIMIT: i32 = 9;

fn init_level(level: i32) {
    unsafe {
        Clear(BACKGROUND);
        for s in &mut SNAKES {
            s.body.clear();
        }

        // Full perimeter box
        for i in 0..80 {
            set_big_pixel(i, 0, WALLS);
            set_big_pixel(i, 49, WALLS);
        }
        for j in 0..50 {
            set_big_pixel(0, j, WALLS);
            set_big_pixel(79, j, WALLS);
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
                    set_big_pixel(x, 25, WALLS);
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            3 => {
                for y in 10..40 {
                    set_big_pixel(20, y, WALLS);
                    set_big_pixel(60, y, WALLS);
                }
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            4 => {
                for y in 0..30 {
                    set_big_pixel(20, y, WALLS);
                    set_big_pixel(60, 49 - y, WALLS);
                }
                for x in 0..40 {
                    set_big_pixel(x, 36, WALLS);
                    set_big_pixel(81 - x, 13, WALLS);
                }
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            5 => {
                for y in 11..37 {
                    set_big_pixel(21, y, WALLS);
                    set_big_pixel(58, y, WALLS);
                }
                for x in 23..57 {
                    set_big_pixel(x, 9, WALLS);
                    set_big_pixel(x, 38, WALLS);
                }
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            6 => {
                for y in 0..49 {
                    if y >= 19 && y <= 30 { continue; }
                    for x in (10..=70).step_by(10) {
                        set_big_pixel(x, y, WALLS);
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            7 => {
                for y in (1..49).step_by(2) {
                    set_big_pixel(40, y, WALLS);
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            8 => {
                for y in 0..40 {
                    for x in (10..=70).step_by(10) {
                        set_big_pixel(x, if x % 20 == 0 { 49 - y } else { y }, WALLS);
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            9 => {
                for i in 3..47 {
                    set_big_pixel(i + 2, i, WALLS);
                    set_big_pixel(i + 28, i, WALLS);
                }
                SNAKES[0].body.push_front(Point { x: 75, y: 40 });
                SNAKES[1].body.push_front(Point { x: 5, y: 15 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            _ => {
                for y in (1..49).step_by(2) {
                    for j in 0..7 {
                        set_big_pixel(j * 10 + 10, y + if j % 2 == 0 { 0 } else { 1 }, WALLS);
                    }
                }
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }

        for s in &mut SNAKES {
            s.body.push_back(*s.body.back().unwrap());
        }
    }
}

fn erase_snakes() {
    for i in 0..8 {
        for s in unsafe { SNAKES.iter_mut() } {
            for j in (i as usize..s.body.len()).step_by(8) {
                set_big_pixel(s.body[j].x, s.body[j].y, BACKGROUND);
            }
        }
        Wait(35);
    }
}

fn pause(message: &str) {
    LastKey();
    draw_string(80, 113, message, Color::White, true);
    while LastKey() != Some(' ') {
        Wait(1);
    }
    draw_string(80, 113, message, BACKGROUND, true);
    ClearInputBuffer();
}

fn generate_apple() {
    loop {
        let x = RandomInt(1, 79);
        let y = RandomInt(1, 49);
        if get_big_pixel(x, y) != BACKGROUND {
            continue;
        }
        set_big_pixel(x, y, APPLE);
        break;
    }
}

fn read_input_and_delay() -> bool {
    Wait(80);
    let mut desired = [
        unsafe { SNAKES[0].direction },
        unsafe { SNAKES[1].direction },
    ];

    while let Some(k) = LastBufferedKey() {
        match k {
            'p' | 'P' => pause("Paused! Press Space"),
            'w' | 'W' => if unsafe { SNAKES[1].direction } != Direction::Down {
                desired[1] = Direction::Up;
            }
            'a' | 'A' => if unsafe { SNAKES[1].direction } != Direction::Right {
                desired[1] = Direction::Left;
            }
            's' | 'S' => if unsafe { SNAKES[1].direction } != Direction::Up {
                desired[1] = Direction::Down;
            }
            'd' | 'D' => if unsafe { SNAKES[1].direction } != Direction::Left {
                desired[1] = Direction::Right;
            }
            '8' => if unsafe { SNAKES[0].direction } != Direction::Down {
                desired[0] = Direction::Up;
            }
            '4' => if unsafe { SNAKES[0].direction } != Direction::Right {
                desired[0] = Direction::Left;
            }
            '2' => if unsafe { SNAKES[0].direction } != Direction::Up {
                desired[0] = Direction::Down;
            }
            '6' => if unsafe { SNAKES[0].direction } != Direction::Left {
                desired[0] = Direction::Right;
            }
            '\x1b' => {
                CloseWindow();
                return true;
            }
            _ => (),
        }

        if desired[0] != unsafe { SNAKES[0].direction } || desired[1] != unsafe { SNAKES[1].direction } {
            break;
        }
    }

    unsafe {
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
    DrawRectangle(0, 0, 320, 10, BACKGROUND, true);

    if player_count > 0 {
        draw_string(5, 3, &format!("{} <-- Sammy", unsafe { SNAKES[0].score }), unsafe { SNAKES[0].color }, false);
    }
    if player_count > 1 {
        draw_string(110, 3, &format!("Jake --> {}", unsafe { SNAKES[1].score }), unsafe { SNAKES[1].color }, false);
    }

    for i in 0..unsafe { max(SNAKES[0].lives - 1, 0) } {
        DrawRectangle(2 + i * 6, 114, 4, 2, unsafe { SNAKES[0].color }, true);
    }
    for i in 0..unsafe { max(SNAKES[1].lives - 1, 0) } {
        DrawRectangle(154 - i * 6, 114, 4, 2, unsafe { SNAKES[1].color }, true);
    }
}

fn game_over_play_again() -> bool {
    LastKey();
    DrawRectangle(41, 41, 80, 40, Color::Black, true);
    DrawRectangle(40, 40, 80, 40, Color::DarkGray, false);
    DrawRectangle(40, 40, 80, 40, Color::Transparent, false);
    draw_string(80, 50, "G A M E   O V E R", Color::LightGray, true);
    draw_string(80, 66, "Play Again? (Y/N)", Color::White, true);

    loop {
        match LastKey() {
            Some('y') | Some('Y') => return true,
            Some('n') | Some('N') => return false,
            _ => Wait(1),
        }
    }
}

fn how_many_players() -> i32 {
    LastKey();
    draw_string(80, 40, "How many players (1 or 2)?", Color::LightGray, true);

    loop {
        match LastKey() {
            Some('1') => return 1,
            Some('2') => return 2,
            _ => Wait(1),
        }
    }
}

const MS_PER_MINUTE: i32 = 60000;
const BPM: i32 = 160;
const BEATS_PER_MS: i32 = MS_PER_MINUTE / BPM;

const DURATION4: i32 = BEATS_PER_MS * 1 / 1;
const DURATION8: i32 = BEATS_PER_MS * 1 / 2;
const DURATION10: i32 = BEATS_PER_MS * 2 / 5;
const DURATION16: i32 = BEATS_PER_MS * 1 / 4;
const DURATION20: i32 = BEATS_PER_MS * 1 / 5;
const DURATION32: i32 = BEATS_PER_MS * 1 / 8;

fn play_title_music() {
    for &n in [0, 48, 50, 52, 50, 48, 50].iter() {
        PlayMusic(n, DURATION8);
    }
    for &n in [52, 48, 48].iter() {
        PlayMusic(n, DURATION4);
    }
}

fn play_level_start() {
    for &n in [60, 62, 64, 62, 60, 62].iter() {
        PlayMusic(n, DURATION20);
    }
    for &n in [64, 60, 60].iter() {
        PlayMusic(n, DURATION10);
    }
}

fn play_apple_pickup() {
    for &n in [48, 48, 48, 52].iter() {
        PlayMusic(n, DURATION16);
    }
}

fn play_snake_dead() {
    for &n in [36, 37, 39, 36, 37, 34, 32].iter() {
        PlayMusic(n, DURATION32);
    }
}

fn run() {
    clear_and_draw_introduction();

    play_title_music();

    while LastKey().is_none() {
        Wait(1);
    }

    initialize_snakes();

    game_loop();
}

fn clear_and_draw_introduction() {
    draw_string(80, 10, "C + +    N i b b l e s", TEXT, true);
    DrawLine(47, 17, 111, 17, 1, Color::DarkGray);
    draw_string(80, 26, "Eat apples while avoiding walls & snakes.", Color::LightGray, true);
    draw_string(80, 32, "The more you eat, the longer you become.", Color::LightGray, true);
    draw_string(80, 50, "Player 1: Arrow keys", Color::Yellow, true);
    draw_string(80, 56, "Player 2: W, A, S, D", Color::LightMagenta, true);
    draw_string(80, 62, "P to Pause", Color::LightGray, true);
    draw_string(80, 90, "Press any key to continue", TEXT, true);
}

fn initialize_snakes() {
    unsafe {
        SNAKES[0].color = Color::Yellow;
        SNAKES[1].color = Color::LightMagenta;
        SNAKES[0].lives = 5;
        SNAKES[1].lives = 5;
        SNAKES[0].score = 0;
        SNAKES[1].score = 0;
    }
}

fn game_loop() {
    loop {
        Clear(BACKGROUND);

        let players = how_many_players();
        adjust_lives_for_one_player_mode(players);

        let mut level = 1;
        let mut pickup_count = 0;

        while unsafe { SNAKES[0].lives > 0 || SNAKES[1].lives > 0 } {
            init_level(level);
            draw_scores(players);
            pause(&format!("Level {}, push space", level));
            play_level_start();

            generate_apple();

            let mut dead = false;
            while !dead && pickup_count < PICKUP_LIMIT {
                if read_input_and_delay() {
                    return;
                }

                process_snakes_movement(&mut dead, &mut pickup_count, players);

                adjust_level_and_reset_pickup_count(&mut level, &mut pickup_count);
                erase_snakes();
            }
        }

        if !game_over_play_again() {
            CloseWindow();
            break;
        }
    }
}

fn adjust_lives_for_one_player_mode(players: i32) {
    unsafe {
        if players == 1 {
            SNAKES[1].lives = 0;
        }
    }
}

fn process_snakes_movement(dead: &mut bool, pickup_count: &mut i32, players: i32) {
    for s in unsafe { SNAKES.iter_mut() } {
        if s.lives == 0 {
            continue;
        }
        
        let new_head = advance_point(s.body.front().cloned().unwrap(), s.direction);
        let hit = get_big_pixel(new_head.x, new_head.y);

        if hit == APPLE {
            play_apple_pickup();
            *pickup_count += 1;
            if *pickup_count < PICKUP_LIMIT {
                generate_apple();
            }
            s.score += *pickup_count;
            let new_segments = *pickup_count * 4;

            for _ in 0..new_segments {
                s.body.push_back(*s.body.back().unwrap());
            }

            draw_scores(players);
        } else if hit != BACKGROUND {
            play_snake_dead();
            s.lives -= 1;
            *dead = true;
            break;
        }

        set_big_pixel(new_head.x, new_head.y, s.color);
        s.body.push_front(new_head);

        let old_tail = s.body.pop_back().unwrap();
        set_big_pixel(old_tail.x, old_tail.y, BACKGROUND);
    }
}

fn adjust_level_and_reset_pickup_count(level: &mut i32, pickup_count: &mut i32) {
    if *pickup_count >= PICKUP_LIMIT {
        *level += 1;
        *pickup_count = 0;
    }
}

fn main() {
    run();
}