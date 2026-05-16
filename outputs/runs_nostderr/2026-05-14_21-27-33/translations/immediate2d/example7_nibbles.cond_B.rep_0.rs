use std::collections::VecDeque;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq)]
struct Color(f32, f32, f32, f32);

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

static mut SNAKES: [Snake; 2] = [
    Snake {
        body: VecDeque::new(),
        direction: Direction::Right,
        lives: 0,
        score: 0,
        color: Color(0.0, 0.0, 0.0, 0.0),
    },
    Snake {
        body: VecDeque::new(),
        direction: Direction::Left,
        lives: 0,
        score: 0,
        color: Color(0.0, 0.0, 0.0, 0.0),
    },
];

fn draw_pixel(_x: i32, _y: i32, _c: &Color) {
    // Placeholder for the drawing functionality
}

fn read_pixel(_x: i32, _y: i32) -> Color {
    Color(0.0, 0.0, 0.0, 0.0)
}

const LIGHT_RED: Color = Color(1.0, 0.0, 0.0, 1.0);
const BLUE: Color = Color(0.0, 0.0, 1.0, 1.0);
const WHITE: Color = Color(1.0, 1.0, 1.0, 1.0);
const YELLOW: Color = Color(1.0, 1.0, 0.0, 1.0);
const LIGHT_MAGENTA: Color = Color(1.0, 0.0, 1.0, 1.0);
const BACKGROUND: Color = Color(0.0, 0.0, 0.0, 1.0);
const APPLE: Color = Color(0.0, 1.0, 0.0, 1.0);
const BLACK: Color = Color(0.0, 0.0, 0.0, 1.0);
const DARK_GRAY: Color = Color(0.2, 0.2, 0.2, 1.0);
const TRANSPARENT: Color = Color(0.0, 0.0, 0.0, 0.0);
const LIGHT_GRAY: Color = Color(0.8, 0.8, 0.8, 1.0);

fn set_big_pixel(x: i32, y: i32, c: Color) {
    draw_pixel(x * 2, y * 2 + 10, &c);
    draw_pixel(x * 2 + 1, y * 2 + 10, &c);
    draw_pixel(x * 2, y * 2 + 11, &c);
    draw_pixel(x * 2 + 1, y * 2 + 11, &c);
}

fn get_big_pixel(x: i32, y: i32) -> Color {
    read_pixel(x * 2, y * 2 + 10)
}

fn draw_string(x: i32, y: i32, s: &str, c: Color, centered: bool) {
    const FONT: [u32; 64] = [
        0x10000000, 0x10000017, 0x30000C03, 0x50AFABEA, 0x509AFEB2, 0x30004C99, 0x400A26AA, 0x10000003,
        0x2000022E, 0x200001D1, 0x30001445, 0x300011C4, 0x10000018, 0x30001084, 0x10000010, 0x30000C98,
        0x30003A2E, 0x300043F2, 0x30004AB9, 0x30006EB1, 0x30007C87, 0x300026B7, 0x300076BF, 0x30007C21,
        0x30006EBB, 0x30007EB7, 0x1000000A, 0x1000001A, 0x30004544, 0x4005294A, 0x30001151, 0x30000AA1,
        0x506ADE2E, 0x300078BE, 0x30002ABF, 0x3000462E, 0x30003A3F, 0x300046BF, 0x300004BF, 0x3000662E,
        0x30007C9F, 0x1000001F, 0x30003E08, 0x30006C9F, 0x3000421F, 0x51F1105F, 0x51F4105F, 0x4007462E,
        0x300008BF, 0x400F662E, 0x300068BF, 0x300026B2, 0x300007E1, 0x30007E1F, 0x30003E0F, 0x50F8320F,
        0x30006C9B, 0x30000F83, 0x30004EB9, 0x30004764, 0x1000001F, 0x30001371, 0x50441044, 0x00000000,
    ];

    let mut x_offset = x;
    if centered {
        x_offset -= s.chars().map(|ch| {
            if ch < ' ' || ch > 127 as char {
                0
            } else {
                ((FONT[ch as usize - 32] >> 28) + 1) as i32
            }
        }).sum::<i32>() / 2;
    }

    for i in s.chars() {
        if i < ' ' || i > (127 as char) {
            continue;
        }
        let mut glyph = FONT[i as usize - 32];
        let width = glyph >> 28;
        for u in x_offset..x_offset + width as i32 {
            for v in y..y + 5 {
                if (glyph & 1) == 1 {
                    draw_pixel(u, v, &c);
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
        for s in &mut SNAKES {
            s.body.clear();
        }
    }

    for i in 0..80 {
        set_big_pixel(i, 0, LIGHT_RED);
        set_big_pixel(i, 49, LIGHT_RED);
    }
    for j in 0..50 {
        set_big_pixel(0, j, LIGHT_RED);
        set_big_pixel(79, j, LIGHT_RED);
    }

    unsafe {
        match level {
            1 => {
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Right;
                SNAKES[1].direction = Direction::Left;
            }
            2 => {
                (20..60).for_each(|x| set_big_pixel(x, 25, LIGHT_RED));
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            3 => {
                (10..40).for_each(|y| {
                    set_big_pixel(20, y, LIGHT_RED);
                    set_big_pixel(60, y, LIGHT_RED);
                });
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            4 => {
                (0..30).for_each(|y| {
                    set_big_pixel(20, y, LIGHT_RED);
                    set_big_pixel(60, 49 - y, LIGHT_RED);
                });
                (0..40).for_each(|x| {
                    set_big_pixel(x, 36, LIGHT_RED);
                    set_big_pixel(81 - x, 13, LIGHT_RED);
                });
                SNAKES[0].body.push_front(Point { x: 60, y: 7 });
                SNAKES[1].body.push_front(Point { x: 20, y: 43 });
                SNAKES[0].direction = Direction::Left;
                SNAKES[1].direction = Direction::Right;
            }
            5 => {
                (11..37).for_each(|y| {
                    set_big_pixel(21, y, LIGHT_RED);
                    set_big_pixel(58, y, LIGHT_RED);
                });
                (23..57).for_each(|x| {
                    set_big_pixel(x, 9, LIGHT_RED);
                    set_big_pixel(x, 38, LIGHT_RED);
                });
                SNAKES[0].body.push_front(Point { x: 50, y: 25 });
                SNAKES[1].body.push_front(Point { x: 30, y: 25 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            6 => {
                (0..49).for_each(|y| {
                    if (19..=30).contains(&y) {
                        return;
                    }
                    (10..=70).step_by(10).for_each(|x| set_big_pixel(x, y, LIGHT_RED));
                });
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            7 => {
                (1..49).step_by(2).for_each(|y| set_big_pixel(40, y, LIGHT_RED));
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            8 => {
                (0..40).for_each(|y| {
                    (10..=70)
                        .step_by(10)
                        .for_each(|x| set_big_pixel(x, if x % 20 == 0 { 49 - y } else { y }, LIGHT_RED));
                });
                SNAKES[0].body.push_front(Point { x: 65, y: 7 });
                SNAKES[1].body.push_front(Point { x: 15, y: 43 });
                SNAKES[0].direction = Direction::Down;
                SNAKES[1].direction = Direction::Up;
            }
            9 => {
                (3..47).for_each(|i| {
                    set_big_pixel(i + 2, i, LIGHT_RED);
                    set_big_pixel(i + 28, i, LIGHT_RED);
                });
                SNAKES[0].body.push_front(Point { x: 75, y: 40 });
                SNAKES[1].body.push_front(Point { x: 5, y: 15 });
                SNAKES[0].direction = Direction::Up;
                SNAKES[1].direction = Direction::Down;
            }
            _ => {
                (1..49).step_by(2).for_each(|y| {
                    (0..7).for_each(|j| {
                        set_big_pixel(j * 10 + 10, y + if j % 2 == 0 { 0 } else { 1 }, LIGHT_RED);
                    });
                });

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
        unsafe {
            for s in &mut SNAKES {
                for j in (i..s.body.len()).step_by(8) {
                    set_big_pixel(s.body[j].x, s.body[j].y, BACKGROUND);
                }
            }
        }
    }
}

fn pause(message: &str) {
    let last_key = || -> char { ' ' };

    draw_string(80, 113, message, WHITE, true);
    while last_key() != ' ' {}
    draw_string(80, 113, message, BACKGROUND, true);
}

fn generate_apple() {
    loop {
        let x = (rand_num() % 79 + 1) as i32;
        let y = (rand_num() % 49 + 1) as i32;

        if get_big_pixel(x, y) != BACKGROUND {
            continue;
        }

        set_big_pixel(x, y, APPLE);
        return;
    }
}

fn rand_num() -> u32 {
    static mut SEED: u32 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        (SEED / 65536) % 32768
    }
}

fn read_input_and_delay() -> bool {
    let wait = |_: i32| {};
    let last_buffered_key = || -> Option<char> { None };

    wait(80);

    let mut desired = unsafe { [SNAKES[0].direction, SNAKES[1].direction] };
    while let Some(k) = last_buffered_key().map(|ch| {
        let valid_keys = ['p', 'w', 'a', 's', 'd', 'u', 'l', 'r', 'e'];
        if valid_keys.contains(&ch.to_ascii_lowercase()) {
            Some(ch.to_ascii_lowercase())
        } else {
            None
        }
    }).flatten() {
        match k {
            'p' => pause("Paused! Press Space"),

            'w' if unsafe { SNAKES[1].direction } != Direction::Down => desired[1] = Direction::Up,
            'a' if unsafe { SNAKES[1].direction } != Direction::Right => desired[1] = Direction::Left,
            's' if unsafe { SNAKES[1].direction } != Direction::Up => desired[1] = Direction::Down,
            'd' if unsafe { SNAKES[1].direction } != Direction::Left => desired[1] = Direction::Right,

            'u' if unsafe { SNAKES[0].direction } != Direction::Down => desired[0] = Direction::Up,
            'l' if unsafe { SNAKES[0].direction } != Direction::Right => desired[0] = Direction::Left,
            'd' if unsafe { SNAKES[0].direction } != Direction::Up => desired[0] = Direction::Down,
            'r' if unsafe { SNAKES[0].direction } != Direction::Left => desired[0] = Direction::Right,

            'e' => return true,
            _ => {}
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
    let draw_rectangle = |_, _, _, _, _, _| {};

    draw_rectangle(0, 0, 160, 10, BACKGROUND, true);

    unsafe {
        if player_count > 0 {
            draw_string(5, 3, &format!("{} <-- Sammy", SNAKES[0].score), SNAKES[0].color, false);
        }
        if player_count > 1 {
            draw_string(110, 3, &format!("Jake --> {}", SNAKES[1].score), SNAKES[1].color, false);
        }

        for i in 0..SNAKES[0].lives - 1 {
            draw_rectangle(2 + i * 6, 114, 4, 2, SNAKES[0].color, false);
        }
        for i in 0..SNAKES[1].lives - 1 {
            draw_rectangle(154 - i * 6, 114, 4, 2, SNAKES[1].color, false);
        }
    }
}

fn game_over_play_again() -> bool {
    let last_key = || -> char { ' ' };

    let draw_rectangle = |_, _, _, _, _, _| {};

    draw_rectangle(41, 41, 80, 40, BLACK, true);
    draw_rectangle(40, 40, 80, 40, DARK_GRAY, true);
    draw_rectangle(40, 40, 80, 40, TRANSPARENT, true);
    draw_string(80, 50, "G A M E   O V E R", LIGHT_GRAY, true);
    draw_string(80, 66, "Play Again? (Y/N)", WHITE, true);

    loop {
        match last_key().to_ascii_lowercase() {
            'y' => return true,
            'n' => return false,
            _ => {}
        }
    }
}

fn how_many_players() -> i32 {
    let last_key = || -> char { ' ' };

    draw_string(80, 40, "How many players (1 or 2)?", LIGHT_GRAY, true);

    loop {
        match last_key() {
            '1' => return 1,
            '2' => return 2,
            _ => {}
        }
    }
}

const MS_PER_MINUTE: i32 = 60000;
const BPM: i32 = 160;
const BEATS_PER_MS: i32 = MS_PER_MINUTE / BPM;

const DURATION_4: i32 = BEATS_PER_MS;
const DURATION_8: i32 = BEATS_PER_MS / 2;
const DURATION_10: i32 = BEATS_PER_MS * 2 / 5;
const DURATION_16: i32 = BEATS_PER_MS / 4;
const DURATION_20: i32 = BEATS_PER_MS / 5;
const DURATION_32: i32 = BEATS_PER_MS / 8;

fn play_music(_note: i32, _duration: i32) {}

fn play_title_music() {
    const NOTES: [(i32, i32); 10] = [
        (0, DURATION_8), (48, DURATION_8), (50, DURATION_8), (52, DURATION_8),
        (50, DURATION_8), (48, DURATION_8), (50, DURATION_8), (52, DURATION_4),
        (48, DURATION_4), (48, DURATION_4),
    ];
    for &(note, duration) in &NOTES {
        play_music(note, duration);
    }
}

fn play_level_start() {
    const NOTES: [(i32, i32); 9] = [
        (60, DURATION_20), (62, DURATION_20), (64, DURATION_20), (62, DURATION_20),
        (60, DURATION_20), (62, DURATION_20), (64, DURATION_10),
        (60, DURATION_10), (60, DURATION_10),
    ];
    for &(note, duration) in &NOTES {
        play_music(note, duration);
    }
}

fn play_apple_pickup() {
    const NOTES: [(i32, i32); 4] = [
        (48, DURATION_16), (48, DURATION_16), (48, DURATION_16), (52, DURATION_16),
    ];
    for &(note, duration) in &NOTES {
        play_music(note, duration);
    }
}

fn play_snake_dead() {
    const NOTES: [(i32, i32); 7] = [
        (36, DURATION_32), (37, DURATION_32), (39, DURATION_32),
        (36, DURATION_32), (37, DURATION_32), (34, DURATION_32), (32, DURATION_32),
    ];
    for &(note, duration) in &NOTES {
        play_music(note, duration);
    }
}

fn run() {
    draw_string(80, 10, "C + +    N i b b l e s", WHITE, true);
    let draw_line = |_, _, _, _, _, _| {};
    let wait = |_: i32| {};
    let last_key = || -> char { ' ' };

    draw_line(47, 17, 111, 17, 1, DARK_GRAY);
    draw_string(80, 26, "Eat apples while avoiding walls & snakes.", LIGHT_GRAY, true);
    draw_string(80, 32, "The more you eat, the longer you become.", LIGHT_GRAY, true);
    draw_string(80, 50, "Player 1: Arrow keys", YELLOW, true);
    draw_string(80, 56, "Player 2: W, A, S, D", LIGHT_MAGENTA, true);
    draw_string(80, 62, "P to Pause", LIGHT_GRAY, true);
    draw_string(80, 90, "Press any key to continue", WHITE, true);

    play_title_music();
    while last_key() == ' ' {
        wait(1);
    }

    unsafe {
        SNAKES[0].color = YELLOW;
        SNAKES[1].color = LIGHT_MAGENTA;
    }

    loop {
        let clear = || {};

        clear();
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

                    for s in &mut SNAKES {
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
                                s.body.push_back(s.body.back().copied().unwrap());
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

                        let old_tail = s.body.pop_back().unwrap();
                        set_big_pixel(old_tail.x, old_tail.y, BACKGROUND);
                    }
                }

                if pickup_count >= PICKUP_LIMIT {
                    level += 1;
                    pickup_count = 0;
                }

                erase_snakes();
            }

            if !game_over_play_again() {
                return;
            }
        }
    }
}

fn main() {
    run();
}