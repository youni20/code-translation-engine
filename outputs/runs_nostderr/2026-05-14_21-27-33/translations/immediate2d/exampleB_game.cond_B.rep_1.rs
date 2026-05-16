use std::collections::HashMap;

const TILE_S: i32 = 10;
const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

// TileS should divide Width and Height evenly

// Width and Height in tiles
const TILE_W: i32 = WIDTH / TILE_S;
const TILE_H: i32 = HEIGHT / TILE_S;

// Enum for tile identifiers
#[derive(Copy, Clone)]
enum TileId {
    Smile,
    Door,
    Wall,
    Trigger,
    Coin,
    BugH,
    BugV,
    Floor,
}

static IMAGE_RESOURCE_NAMES: [&str; 8] = [
    "smile", "door", "wall", "trigger", "coin", "bugH", "bugV", "floor",
];
static TILE_LOOKUP: &str = "@!#1$-| ";

// Type alias for Tiles
type Tile = char;

fn lookup_tile_id(c: char) -> TileId {
    if c.is_digit(10) {
        return TileId::Trigger;
    }
    match TILE_LOOKUP.find(c) {
        Some(index) => unsafe { std::mem::transmute(index as u8) },
        None => TileId::Floor,
    }
}

// Action struct to define a single unit of change
struct Action {
    x: i32,
    y: i32,
    t: Tile,
}

type TriggerTemplate = Vec<Action>;

const CENTI_PIXELS_PER_PIXEL: i32 = 100;

// Bug struct, representing moving elements
struct Bug {
    x: i32,
    y: i32,
    id: TileId,
    increasing: bool,
    wait: i32,
}

impl Bug {
    fn new(x: i32, y: i32, id: TileId) -> Self {
        Bug {
            x,
            y,
            id,
            increasing: true,
            wait: 0,
        }
    }

    fn distance_to(&self, t_x: i32, t_y: i32) -> i32 {
        let c_x = t_x * TILE_S * CENTI_PIXELS_PER_PIXEL;
        let c_y = t_y * TILE_S * CENTI_PIXELS_PER_PIXEL;
        (c_x - self.x).abs() + (c_y - self.y).abs()
    }
}

struct Player {
    x: i32,
    y: i32,
}

struct Level {
    map: [[Tile; TILE_W as usize]; TILE_H as usize],
    bugs: Vec<Bug>,
    players: Vec<Player>,
    templates: [TriggerTemplate; Level::TEMPLATE_COUNT],
}

impl Level {
    const TEMPLATE_COUNT: usize = 10;

    fn new() -> Self {
        Level {
            map: [[' '; TILE_W as usize]; TILE_H as usize],
            bugs: vec![],
            players: vec![],
            templates: Default::default(),
        }
    }

    fn apply(&mut self, action: &Action) {
        self.map[action.y as usize][action.x as usize] = action.t;
    }

    fn from_text(text: &str, level_number: u32) -> Self {
        let mut level = Self::new();
        let mut interest = HashMap::new();

        let tokenize = |c: char| -> Token {
            match c {
                '~' => Token::Tilde,
                ':' => Token::Colon,
                '.' => Token::Period,
                '0'..='9' => Token::Number,
                'a'..='z' | 'A'..='Z' => Token::Letter,
                _ if TILE_LOOKUP.contains(c) => Token::Tile,
                _ => Token::Unknown,
            }
        };

        // Process the map from the string text
        let mut lines = text.lines();
        for y in 0..TILE_H {
            let line = lines.next().unwrap_or("");
            for (x, c) in line.chars().enumerate().take(TILE_W as usize) {
                let tile = &mut level.map[y as usize][x];
                match tokenize(c) {
                    Token::Tile => *tile = TILE_LOOKUP
                        .chars()
                        .nth(lookup_tile_id(c) as usize)
                        .unwrap_or(' '),
                    Token::Number => {
                        *tile = c;
                        interest.entry(c).or_insert_with(Vec::new).push((x as i32, y));
                    }
                    Token::Letter => {
                        *tile = '#'; // Consider '#' as a placeholder
                        interest.entry(c).or_insert_with(Vec::new).push((x as i32, y));
                    }
                    _ => (),
                }
            }
        }

        // Parsing trigger templates
        enum State {
            Name,
            Target,
            Value,
        }
        let mut state = State::Name;
        let mut name = '0';
        let mut target = '0';
        let mut text_iter = text.chars().peekable();

        while let Some(&c) = text_iter.peek() {
            text_iter.next();

            match state {
                State::Name => {
                    if let Token::Number | Token::Tilde = tokenize(c) {
                        state = State::Target;
                        name = c;
                    }
                }
                State::Target => {
                    if tokenize(c) == Token::Period {
                        state = State::Name;
                    } else if let Token::Letter | Token::Number = tokenize(c) {
                        state = State::Value;
                        target = c;
                    }
                }
                State::Value => {
                    if let Token::Tile | Token::Number = tokenize(c) {
                        if let Some(points) = interest.get(&target) {
                            for &(x, y) in points {
                                let action = Action {
                                    x,
                                    y,
                                    t: TILE_LOOKUP
                                        .chars()
                                        .nth(TileId::Floor as usize)
                                        .unwrap_or(' '),
                                };
                                if name == '~' {
                                    level.apply(&action);
                                } else {
                                    level.templates[name as usize - '0' as usize].push(action);
                                }
                            }
                        }
                        state = State::Target;
                    }
                }
            }
        }

        level
    }
}

#[derive(PartialEq)]
enum Token {
    Number,
    Tilde,
    Colon,
    Letter,
    Tile,
    Period,
    Unknown,
}

fn play_sfx(t: TileId) {
    match t {
        TileId::Floor => return,
        TileId::Coin => {
            play_music(83, 60);
            play_music(88, 150);
        }
        TileId::BugH | TileId::BugV => {
            for n in &[36, 37, 39, 36, 37, 34, 32] {
                play_music(*n, 60);
            }
        }
        TileId::Door => {
            for (n, duration) in &[
                (49, 303),
                (50, 110),
                (49, 211),
                (47, 182),
                (45, 200),
                (45, 87),
                (49, 54),
                (52, 45),
                (57, 117),
            ] {
                play_music(*n, *duration);
            }
        }
        TileId::Trigger => {
            for (n, duration) in &[(69, 43), (73, 27), (66, 23), (81, 117)] {
                play_music(*n, *duration);
            }
        }
        _ => {
            for n in &[37, 34, 32] {
                play_music(*n, 20);
            }
        }
    };
}

fn draw_title_card(text: &str, c: Color) {
    clear();
    draw_string(WIDTH / 2, HEIGHT / 2 - 12, text, "Arial", 12, c, true);
    present();
}

// Implement a placeholder function for playing music to avoid Rust compilation errors
fn play_music(_note: i32, _duration: i32) {}

// Implement a placeholder function to clear the graphics for compiling
fn clear() {}

// Implement a placeholder function for drawing strings
fn draw_string(
    _x: i32,
    _y: i32,
    _text: &str,
    _font: &str,
    _size: i32,
    _color: Color,
    _align_center: bool,
) {
}

// Implement a placeholder function to present the graphics frame
fn present() {}

// Placeholder Color struct
#[derive(Copy, Clone)]
struct Color;

// Main game run function demonstrating just a portion due to space constraints
fn main() {
    // Placeholder game loop and simulation
    let _level = Level::from_text("your level data goes here", 1);
    println!("Level loaded with dimensions {}x{}", TILE_W, TILE_H);

    // Simulating simple movement update and rendering, abstracted here
}