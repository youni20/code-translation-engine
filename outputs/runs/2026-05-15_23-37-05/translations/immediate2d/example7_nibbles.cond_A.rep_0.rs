use std::collections::VecDeque;

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

#[derive(PartialEq, Clone, Copy)]
enum Color {
    LightRed,
    Blue,
    White,
    Yellow,
    LightMagenta,
    DarkGray,
    Black,
    Transparent,
}

static mut SNAKES: [Snake; 2] = [
    Snake {
        body: VecDeque::new(),
        direction: Direction::Right,
        lives: 0,
        score: 0,
        color: Color::Yellow,
    },
    Snake {
        body: VecDeque::new(),
        direction: Direction::Right,
        lives: 0,
        score: 0,
        color: Color::LightMagenta,
    },
];

fn draw_pixel(_x: i32, _y: i32, _c: Color) {
    // Implementation for drawing a single pixel
    // Placeholder function for demonstration purpose
}

fn read_pixel(x: i32, y: i32) -> Color {
    // Implementation for reading a pixel's color
    // Placeholder function for demonstration purpose
    Color::Blue
}

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
    static FONT: [u32; 128 - 32] = [
        0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA,
        0x10000003, 0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084,
        0x10000010, 0x30000C98, 0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87,
        0x300026B7, 0x300076BF, 0x30007C21, 0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A,
        0x30004544, 0x4005294A, 0x30001151, 0x30000AA1, 0x506ADE2E, 0x300078BE, 0x30002ABF,
        0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F,
        0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E, 0x300008BF,
        0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x2000023F, 0x30006083, 0x200003F1, 0x30000822,
        0x30004210, 0x20000041, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF,
        0x300004BF, 0x3000662E, 0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F,
        0x51F1105F, 0x51F4105F, 0x4007462E, 0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2,
        0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F, 0x30006C9B, 0x30000F83, 0x30004EB9,
        0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
    ];

    let width_sum: u32 = if centered {
        s.chars()
            .map(|ch| FONT.get(ch as usize - 32).map_or(0, |glyph| (glyph >> 28) + 1))
            .sum::<u32>()
    } else {
        0
    };

    let mut x_offset = x - (width_sum as i32 / 2);
    for ch in s.chars() {
        if ch < ' ' || ch > '' {
            continue;
        }
        let glyph = FONT.get(ch as usize - 32).unwrap_or(&0);
        let width = (*glyph >> 28) as usize;
        for u in x_offset..x_offset + width as i32 {
            for v in y..y + 5 {
                if glyph & (1 << (v - y)) != 0 {
                    draw_pixel(u, v, c);
                }
            }
        }
        x_offset += width as i32 + 1;
    }
}

const WALLS: Color = Color::LightRed;
const BACKGROUND: Color = Color::Blue;
const TEXT: Color = Color::White;
const APPLE: Color = Color::White;

fn clear(_c: Color) {
    // Placeholder function for demonstration purpose
}

fn wait(_duration: i32) {
    // Placeholder function for demonstration purpose
}

fn last_key() -> Option<char> {
    // Placeholder function for demonstration purpose
    None
}

fn last_buffered_key() -> Option<char> {
    // Placeholder function for demonstration purpose
    None
}

fn close_window() {
    // Placeholder function for demonstration purpose
}

fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _fill_color: Color, _filled: bool) {
    // Placeholder function for demonstration purpose
}

fn draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _thickness: i32, _color: Color) {
    // Placeholder function for demonstration purpose
}

fn random_int(min: i32, _max: i32) -> i32 {
    // Placeholder function for demonstration purpose
    min
}

fn clear_input_buffer() {
    // Placeholder function for demonstration purpose
}

fn play_music(_note: i32, _duration: i32) {
    // Placeholder function for demonstration purpose
}

fn init_level(level: i32) {
    clear(BACKGROUND);
    for s in unsafe { &mut SNAKES } {
        s.body.clear();
    }

    // Draw perimeter box
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
            unsafe {
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Right;
                SNAKES[1].direction = Direction::Left;
            }
        }
        2 => {
            for x in 20..60 {
                set_big_pixel(x, 25, WALLS);
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
        }
        3 => {
            for y in 10..40 {
                set_big_pixel(20, y, WALLS);
                set_big_pixel(60, y, WALLS);
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
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
            unsafe {
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
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
            unsafe {
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
        }
        6 => {
            for y in 0..49 {
                if y >= 19 && y <= 30 {
                    continue;
                }
                for x in (10..=70).step_by(10) {
                    set_big_pixel(x, y, WALLS);
                }
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }
        7 => {
            for y in (1..49).step_by(2) {
                set_big_pixel(40, y, WALLS);
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }
        8 => {
            for y in 0..40 {
                for x in (10..=70).step_by(10) {
                    set_big_pixel(x, if x % 20 == 0 { 49 - y } else { y }, WALLS);
                }
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }
        9 => {
            for i in 3..47 {
                set_big_pixel(i + 2, i, WALLS);
                set_big_pixel(i + 28, i, WALLS);
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 75, y: 40 });
                SNAKES[1].body.push_front(Point { x: 5, y: 15 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
        }
        _ => {
            for y in (1..49).step_by(2) {
                for j in 0..7 {
                    set_big_pixel(j * 10 + 10, y + if j % 2 == 0 { 0 } else { 1 }, WALLS);
                }
            }
            unsafe {
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
        }
    }

    for s in unsafe { &mut SNAKES } {
        if let Some(&back) = s.body.back() {
            s.body.push_back(back);
        }
    }
}

fn erase_snakes() {
    for i in 0..8 {
        for s in unsafe { &SNAKES } {
            for j in (i..s.body.len()).step_by(8) {
                let point = s.body[j];
                set_big_pixel(point.x, point.y, BACKGROUND);
            }
        }
        wait(35);
    }
}

fn pause(message: &str) {
    last_key(); // Discard any pre-existing input

    draw_string(80, 113, message, Color::White, true);
    while last_key() != Some(' ') {
        wait(1);
    }
    draw_string(80, 113, message, BACKGROUND, true);

    clear_input_buffer(); // Wipe out any other keyboard input that happened during the pause
}

fn generate_apple() {
    loop {
        let x = random_int(1, 79);
        let y = random_int(1, 49);
        if get_big_pixel(x, y) != BACKGROUND {
            continue;
        }

        set_big_pixel(x, y, APPLE);
        return;
    }
}

fn read_input_and_delay() -> bool {
    wait(80);

    let mut desired = [
        unsafe { SNAKES[0].direction },
        unsafe { SNAKES[1].direction },
    ];
    while let Some(k) = last_buffered_key() {
        match k {
            'p' | 'P' => pause("Paused! Press Space"),
            'w' | 'W' => {
                if unsafe { SNAKES[1].direction } != Direction::Down {
                    desired[1] = Direction::Up;
                }
            }
            'a' | 'A' => {
                if unsafe { SNAKES[1].direction } != Direction::Right {
                    desired[1] = Direction::Left;
                }
            }
            's' | 'S' => {
                if unsafe { SNAKES[1].direction } != Direction::Up {
                    desired[1] = Direction::Down;
                }
            }
            'd' | 'D' => {
                if unsafe { SNAKES[1].direction } != Direction::Left {
                    desired[1] = Direction::Right;
                }
            }
            _ => (),
        }

        if desired[0] != unsafe { SNAKES[0].direction }
            || desired[1] != unsafe { SNAKES[1].direction }
        {
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
    draw_rectangle(0, 0, 160, 10, BACKGROUND, true);

    if player_count > 0 {
        draw_string(
            5,
            3,
            &format!(
                "{} <-- Sammy",
                unsafe { SNAKES[0].score.to_string() }
            ),
            unsafe { SNAKES[0].color },
            false,
        );
    }
    if player_count > 1 {
        draw_string(
            110,
            3,
            &format!("Jake --> {}", unsafe { SNAKES[1].score.to_string() }),
            unsafe { SNAKES[1].color },
            false,
        );
    }

    for i in 0..unsafe { SNAKES[0].lives - 1 } {
        draw_rectangle(2 + i * 6, 114, 4, 2, unsafe { SNAKES[0].color }, true);
    }
    for i in 0..unsafe { SNAKES[1].lives - 1 } {
        draw_rectangle(154 - i * 6, 114, 4, 2, unsafe { SNAKES[1].color }, true);
    }
}

fn game_over_play_again() -> bool {
    last_key(); // Discard any pre-existing input

    draw_rectangle(41, 41, 80, 40, Color::Black, true);
    draw_rectangle(40, 40, 80, 40, Color::DarkGray, true);
    draw_rectangle(40, 40, 80, 40, Color::Transparent, true);
    draw_string(80, 50, "G A M E   O V E R", Color::White, true);
    draw_string(80, 66, "Play Again? (Y/N)", Color::White, true);

    loop {
        if let Some(c) = last_key() {
            if c == 'y' || c == 'Y' {
                return true;
            }
            if c == 'n' || c == 'N' {
                return false;
            }
        }

        wait(1);
    }
}

fn how_many_players() -> i32 {
    last_key(); // Discard any pre-existing input

    draw_string(80, 40, "How many players (1 or 2)?", Color::White, true);

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

// Music timing constants
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
    for &n in &[0, 48, 50, 52, 50, 48, 50] {
        play_music(n, DURATION8);
    }
    for &n in &[52, 48, 48] {
        play_music(n, DURATION4);
    }
}

fn play_level_start() {
    for &n in &[60, 62, 64, 62, 60, 62] {
        play_music(n, DURATION20);
    }
    for &n in &[64, 60, 60] {
        play_music(n, DURATION10);
    }
}

fn play_apple_pickup() {
    for &n in &[48, 48, 48, 52] {
        play_music(n, DURATION16);
    }
}

fn play_snake_dead() {
    for &n in &[36, 37, 39, 36, 37, 34, 32] {
        play_music(n, DURATION32);
    }
}

fn run() {
    draw_string(80, 10, "R u s t    N i b b l e s", Color::White, true);
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
        clear(Color::Black);
        let players = how_many_players();

        unsafe {
            SNAKES[0].lives = 5;
            SNAKES[1].lives = 5;
            SNAKES[0].score = 0;
            SNAKES[1].score = 0;

            if players == 1 {
                SNAKES[1].lives = 0;
            }
        }

        let mut level = 1;
        let mut pickup_count = 0;
        const PICKUP_LIMIT: i32 = 9;

        while unsafe { SNAKES[0].lives > 0 || SNAKES[1].lives > 0 } {
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

                for s in unsafe { &mut SNAKES } {
                    if s.lives == 0 {
                        continue;
                    }

                    let new_head = advance_point(s.body.front().copied().unwrap(), s.direction);

                    let hit = get_big_pixel(new_head.x, new_head.y);
                    if hit == APPLE {
                        play_apple_pickup();
                        pickup_count += 1;
                        if pickup_count < PICKUP_LIMIT {
                            generate_apple();
                        }

                        s.score += pickup_count;
                        let new_segments = pickup_count * 4;
                        for _ in 0..new_segments {
                            if let Some(&back) = s.body.back() {
                                s.body.push_back(back);
                            }
                        }
                        draw_scores(players);
                    } else if hit != BACKGROUND {
                        play_snake_dead();
                        s.lives -= 1;
                        dead = true;
                        break;
                    }

                    set_big_pixel(new_head.x, new_head.y, s.color);
                    s.body.push_front(new_head); 

                    if let Some(old_tail) = s.body.pop_back() {
                        set_big_pixel(old_tail.x, old_tail.y, BACKGROUND);
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
            break;
        }
    }

    close_window();
}

fn main() {
    run();
}