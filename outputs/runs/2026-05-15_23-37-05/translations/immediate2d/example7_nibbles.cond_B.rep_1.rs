use std::collections::VecDeque;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq)]
struct Color(u8, u8, u8);

struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
    lives: i32,
    score: i32,
    color: Color,
}

impl Default for Snake {
    fn default() -> Self {
        Snake {
            body: VecDeque::new(),
            direction: Direction::Up,
            lives: 5,
            score: 0,
            color: Color(0, 0, 0),
        }
    }
}

const LIGHT_RED: Color = Color(255, 0, 0);
const BLUE: Color = Color(0, 0, 255);
const WHITE: Color = Color(255, 255, 255);
const YELLOW: Color = Color(255, 255, 0);
const LIGHT_MAGENTA: Color = Color(255, 0, 255);
const DARK_GRAY: Color = Color(169, 169, 169);
const TRANSPARENT: Color = Color(0, 0, 0);
const BLACK: Color = Color(0, 0, 0);

const WALLS: Color = LIGHT_RED;
const BACKGROUND: Color = BLUE;
const TEXT: Color = WHITE;
const APPLE: Color = WHITE;

fn draw_pixel(_x: i32, _y: i32, _c: Color) {
    // Assuming a function to draw a pixel exists
}

fn read_pixel(_x: i32, _y: i32) -> Color {
    // Assuming a function to read a pixel color exists
    Color(0, 0, 0)
}

fn clear(_c: Color) {
    // Assuming a function to clear the screen with a color exists
}

fn draw_rectangle(_x: i32, _y: i32, _width: i32, _height: i32, _border_color: Color, _fill_color: Color) {
    // Assuming a function to draw a filled rectangle
}

fn play_music(_note: i32, _duration: i32) {
    // Assuming a function to play music notes
}

fn close_window() {
    // Assuming a function to close the window
}

fn wait(_ms: i32) {
    // Assuming a function to wait for a specified time
}

fn last_key() -> Option<char> {
    // Assuming a function to get the last pressed key
    None
}

fn last_buffered_key() -> Option<char> {
    // Assuming a function to get the last buffered key
    None
}

// Global state
static mut SNAKES: [Snake; 2] = [
    Snake {
        body: VecDeque::new(),
        direction: Direction::Up,
        lives: 5,
        score: 0,
        color: Color(255, 255, 0),
    },
    Snake {
        body: VecDeque::new(),
        direction: Direction::Up,
        lives: 5,
        score: 0,
        color: Color(255, 0, 255),
    },
];

fn set_big_pixel(x: i32, y: i32, c: Color) {
    draw_pixel(x * 2, y * 2 + 10, c);
    draw_pixel(x * 2 + 1, y * 2 + 10, c);
    draw_pixel(x * 2, y * 2 + 11, c);
    draw_pixel(x * 2 + 1, y * 2 + 11, c);
}

fn get_big_pixel(x: i32, y: i32) -> Color {
    read_pixel(x * 2, y * 2 + 10)
}

fn draw_string(_x: i32, _y: i32, _s: &str, _c: Color, _centered: bool) {
    // Assuming a function to draw a string
}

fn init_level(level: i32) {
    clear(BACKGROUND);
    unsafe {
        for s in &mut SNAKES {
            s.body.clear();
        }
    }

    for i in 0..80 {
        set_big_pixel(i, 0, WALLS);
        set_big_pixel(i, 49, WALLS);
    }
    for j in 0..50 {
        set_big_pixel(0, j, WALLS);
        set_big_pixel(79, j, WALLS);
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
                    if y >= 19 && y <= 30 {
                        continue;
                    }
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
            if let Some(back) = s.body.back() {
                s.body.push_back(back.clone());
            }
        }
    }
}

fn erase_snakes() {
    for _i in 0..8 {
        unsafe {
            for s in &mut SNAKES {
                for j in (0..s.body.len()).step_by(8) {
                    set_big_pixel(s.body[j].x, s.body[j].y, BACKGROUND);
                }
            }
        }
        wait(35);
    }
}

fn pause(_message: &str) {
    last_key();

    draw_string(80, 113, _message, WHITE, true);
    while last_key() != Some(' ') {
        wait(1);
    }
    draw_string(80, 113, _message, BACKGROUND, true);
}

fn generate_apple() {
    loop {
        let x = ((std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() % 79) as i32)
            + 1;

        let y = ((std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() % 49) as i32)
            + 1;

        if get_big_pixel(x, y) != BACKGROUND {
            continue;
        }
        set_big_pixel(x, y, APPLE);
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
                'k' => {
                    if SNAKES[0].direction != Direction::Down {
                        desired[0] = Direction::Up;
                    }
                }
                'j' => {
                    if SNAKES[0].direction != Direction::Right {
                        desired[0] = Direction::Left;
                    }
                }
                'i' => {
                    if SNAKES[0].direction != Direction::Up {
                        desired[0] = Direction::Down;
                    }
                }
                'l' => {
                    if SNAKES[0].direction != Direction::Left {
                        desired[0] = Direction::Right;
                    }
                }
                '\u{1b}' => {
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
    draw_rectangle(0, 0, 160, 10, BACKGROUND, BACKGROUND);
    unsafe {
        if player_count > 0 {
            draw_string(5, 3, &format!("{} <-- Sammy", SNAKES[0].score), SNAKES[0].color, false);
        }
        if player_count > 1 {
            draw_string(110, 3, &format!("Jake --> {}", SNAKES[1].score), SNAKES[1].color, false);
        }

        for i in 0..SNAKES[0].lives - 1 {
            draw_rectangle(2 + i * 6, 114, 4, 2, SNAKES[0].color, SNAKES[0].color);
        }
        for i in 0..SNAKES[1].lives - 1 {
            draw_rectangle(154 - i * 6, 114, 4, 2, SNAKES[1].color, SNAKES[1].color);
        }
    }
}

fn game_over_play_again() -> bool {
    last_key();
    draw_rectangle(41, 41, 80, 40, BLACK, DARK_GRAY);
    draw_string(80, 50, "G A M E   O V E R", DARK_GRAY, true);
    draw_string(80, 66, "Play Again? (Y/N)", WHITE, true);

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
    last_key();
    draw_string(80, 40, "How many players (1 or 2)?", WHITE, true);

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
    const DUR_8TH: i32 = 375;
    const DUR_QUARTER: i32 = 750;
    for n in &[0, 48, 50, 52, 50, 48, 50] {
        play_music(*n, DUR_8TH);
    }
    for n in &[52, 48, 48] {
        play_music(*n, DUR_QUARTER);
    }
}

fn play_level_start() {
    const DUR_20TH: i32 = 300;
    const DUR_10TH: i32 = 150;
    for n in &[60, 62, 64, 62, 60, 62] {
        play_music(*n, DUR_20TH);
    }
    for n in &[64, 60, 60] {
        play_music(*n, DUR_10TH);
    }
}

fn play_apple_pickup() {
    const DUR_16TH: i32 = 187;
    for n in &[48, 48, 48, 52] {
        play_music(*n, DUR_16TH);
    }
}

fn play_snake_dead() {
    const DUR_32ND: i32 = 93;
    for n in &[36, 37, 39, 36, 37, 34, 32] {
        play_music(*n, DUR_32ND);
    }
}

fn run() {
    draw_string(80, 10, "C + +    N i b b l e s", WHITE, true);

    draw_string(80, 26, "Eat apples while avoiding walls & snakes.", WHITE, true);
    draw_string(80, 32, "The more you eat, the longer you become.", WHITE, true);
    draw_string(80, 50, "Player 1: Arrow keys", YELLOW, true);
    draw_string(80, 56, "Player 2: W, A, S, D", LIGHT_MAGENTA, true);
    draw_string(80, 62, "P to Pause", WHITE, true);
    draw_string(80, 90, "Press any key to continue", WHITE, true);

    play_title_music();
    while last_key().is_none() {
        wait(1);
    }

    // This is the only one-time initialization our snakes need
    unsafe {
        SNAKES[0].color = YELLOW;
        SNAKES[1].color = LIGHT_MAGENTA;
    }

    loop {
        clear(BACKGROUND);
        let players = how_many_players();

        unsafe {
            // Reset scores and lives at the start of each new game
            SNAKES[0].lives = 5;
            SNAKES[1].lives = 5;
            SNAKES[0].score = 0;
            SNAKES[1].score = 0;

            // Kill off player 2 right away if we're in 1-player mode
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

                    // Process each snake's movement
                    for s in &mut SNAKES {
                        // Skip dead snakes
                        if s.lives == 0 {
                            continue;
                        }

                        let new_head = advance_point(s.body.front().unwrap().clone(), s.direction);

                        // Check to see if we've hit anything
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
                                let back = s.body.back().unwrap().clone();
                                s.body.push_back(back);
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
                close_window();
                break;
            }
        }
    }
}

fn main() {
    run();
}