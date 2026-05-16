#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::Debug;
use std::thread::sleep;
use std::time::Duration;

const TILE_S: i32 = 10;
const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const TILE_W: i32 = WIDTH / TILE_S;
const TILE_H: i32 = HEIGHT / TILE_S;
const CENTI_PIXELS_PER_PIXEL: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

static TILE_LOOKUP: [char; 8] = ['@', '!', '#', '1', '$', '-', '|', ' '];

type Tile = char;

fn lookup_tile_id(c: char) -> TileId {
    if c >= '0' && c <= '9' {
        return TileId::Trigger;
    }
    for (i, &ch) in TILE_LOOKUP.iter().enumerate() {
        if c == ch {
            return match i {
                0 => TileId::Smile,
                1 => TileId::Door,
                2 => TileId::Wall,
                3 => TileId::Trigger,
                4 => TileId::Coin,
                5 => TileId::BugH,
                6 => TileId::BugV,
                7 => TileId::Floor,
                _ => TileId::Floor,
            };
        }
    }
    TileId::Floor
}

#[derive(Debug, Clone, Copy)]
struct Action {
    x: i32,
    y: i32,
    t: Tile,
}

type TriggerTemplate = Vec<Action>;

#[derive(Debug)]
struct Bug {
    x: i32,
    y: i32,
    id: TileId,
    increasing: bool,
    wait: i32,
}

impl Bug {
    fn distance_to(&self, t_x: i32, t_y: i32) -> i32 {
        let c_x = t_x * TILE_S * CENTI_PIXELS_PER_PIXEL;
        let c_y = t_y * TILE_S * CENTI_PIXELS_PER_PIXEL;
        (c_x - self.x).abs() + (c_y - self.y).abs()
    }
}

#[derive(Debug)]
struct Level {
    map: [[Tile; TILE_W as usize]; TILE_H as usize],
    bugs: Vec<Bug>,
    players: Vec<Player>,
    templates: [TriggerTemplate; Level::TEMPLATE_COUNT],
}

impl Level {
    const TEMPLATE_COUNT: usize = 10;

    fn new_from_encoded(text: &str, _level_number: i32) -> Result<Self, Box<dyn Error>> {
        // Initialize map with Floor tiles
        let mut map = [[' '; TILE_W as usize]; TILE_H as usize];

        // Other setup code and logic
        let mut interest: HashMap<char, Vec<Point>> = HashMap::new();

        let mut y = 0;
        let mut x = 0;

        let mut iter = text.chars().peekable();
        while y < TILE_H as usize && iter.peek().is_some() {
            while x < TILE_W as usize {
                let c = iter.next().ok_or("Unexpected end of string")?;
                let token = Token::from(c);
                let tile = match token {
                    Token::Tile => lookup_tile_id(c),
                    Token::Number | Token::Letter => {
                        let point = Point { x, y };
                        interest.entry(c).or_insert_with(Vec::new).push(point);
                        lookup_tile_id(c)
                    }
                    _ => TileId::Floor,
                };
                map[y][x] = match tile {
                    TileId::Smile => '@',
                    TileId::Door => '!',
                    TileId::Wall => '#',
                    TileId::Trigger => '1',
                    TileId::Coin => '$',
                    TileId::BugH => '-',
                    TileId::BugV => '|',
                    TileId::Floor => ' ',
                };
                x += 1;
            }
            x = 0;
            y += 1;
        }
        
        Ok(Level {
            map,
            bugs: Vec::new(),
            players: Vec::new(),
            templates: Default::default(),
        })
    }

    fn apply(&mut self, action: &Action) {
        self.map[action.y as usize][action.x as usize] = action.t;
    }
}

#[derive(Debug)]
struct Player {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Eq)]
enum Token {
    Number,
    Tilde,
    Colon,
    Letter,
    Tile,
    Period,
    Unknown,
}

impl From<char> for Token {
    fn from(c: char) -> Self {
        if c == '~' {
            return Token::Tilde;
        }
        if c == ':' {
            return Token::Colon;
        }
        if c == '.' {
            return Token::Period;
        }
        if c.is_digit(10) {
            return Token::Number;
        }
        if c.is_ascii_alphabetic() {
            return Token::Letter;
        }
        for &ch in TILE_LOOKUP.iter() {
            if c == ch {
                return Token::Tile;
            }
        }
        Token::Unknown
    }
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

// Placeholder functions for I/O functions
fn clear() {}

fn draw_image(_x: i32, _y: i32, _image: &str) {}

fn load_image(name: &str) -> String {
    name.to_string()
}

fn last_key() -> Option<char> {
    None
}

fn present() {}

fn wait(ms: u64) {
    sleep(Duration::from_millis(ms));
}

fn use_double_buffering(_enabled: bool) {}

fn draw_title_card(text: &str, _c: i32) {
    clear();
    println!("{}", text);
    present();
}

fn play_sfx(_t: TileId) {}

fn save_to_clipboard(_level: &Level) {}

// Main function to run the game
fn run() {
    use_double_buffering(true);

    // Load images
    let images: Vec<_> = IMAGE_RESOURCE_NAMES
        .iter()
        .map(|&name| load_image(name))
        .collect();

    let mut level_number = 0;
    let mut levels: VecDeque<&str> = VecDeque::new();

    // Main game loop
    while let Some(_text) = levels.pop_front() {
        level_number += 1;
        let level = match Level::new_from_encoded("sample haha", level_number) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error loading level {}: {}", level_number, e);
                continue;
            }
        };

        // Game logic for each level

        while let None = last_key() {
            clear();
            for player in &level.players {
                draw_image(player.x * TILE_S, player.y * TILE_S, &images[TileId::Smile as usize]);
            }
            present();
            wait(16);
        }
    }
}

fn main() {
    run();
}