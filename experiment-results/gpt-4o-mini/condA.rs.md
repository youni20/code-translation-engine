use std::io::{self, BufRead};
use std::time::Instant;

type U64 = u64;

const PAWN: usize = 0;
const KNIGHT: usize = 1;
const BISHOP: usize = 2;
const ROOK: usize = 3;
const QUEEN: usize = 4;
const KING: usize = 5;

const WHITE: usize = 0;
const BLACK: usize = 1;

const INF: i32 = 999999;
const MATE: i32 = 100000;
const MAX_QUIESCENCE_DEPTH: i32 = 6;
const MAX_PLY: usize = 128;

#[derive(Copy, Clone)]
struct TTEntry {
    hash: U64,
    depth: i32,
    score: i32,
    flag: i32, // EXACT, ALPHA, BETA
    best_move: i32,
}

const TT_SIZE: usize = 1 << 20; // 1MB entries
static mut TRANS_TABLE: [TTEntry; TT_SIZE] = [TTEntry { hash: 0, depth: 0, score: 0, flag: 0, best_move: 0 }; TT_SIZE];

const TT_EXACT: i32 = 0;
const TT_ALPHA: i32 = 1;
const TT_BETA: i32 = 2;

struct HistoryTable {
    scores: [[[i32; 64]; 2]; 64], // [side][from][to]
}

impl HistoryTable {
    fn new() -> Self {
        let scores = [[[0; 64]; 2]; 64];
        HistoryTable { scores }
    }

    fn update(&mut self, side: usize, from: usize, to: usize, depth: i32) {
        self.scores[side][from][to] += depth * depth;
        // Aging
        if self.scores[side][from][to] > 100000 {
            for s in 0..2 {
                for f in 0..64 {
                    for t in 0..64 {
                        self.scores[s][f][t] /= 2;
                    }
                }
            }
        }
    }

    fn get(&self, side: usize, from: usize, to: usize) -> i32 {
        self.scores[side][from][to]
    }
}

#[derive(PartialEq)]
struct Move {
    from: i32,
    to: i32,
    score: i32,
    piece: i32,
    captured: i32,
    promo: i32,
}

impl Move {
    pub fn new(from: i32, to: i32, piece: i32, captured: i32, promo: i32) -> Self {
        Move { from, to, score: 0, piece, captured, promo }
    }
}

struct KillerMoves {
    killers: [Option<&'static Move>; 2],
}

impl KillerMoves {
    fn new() -> Self {
        KillerMoves { killers: [None, None] }
    }

    fn update(&mut self, m: &'static Move) {
        if self.killers[0] != Some(m) {
            self.killers[1] = self.killers[0];
            self.killers[0] = Some(m);
        }
    }

    fn is_killer(&self, m: &'static Move) -> bool {
        self.killers[0] == Some(m) || self.killers[1] == Some(m)
    }
}

struct UCIOptions {
    depth: i32,
    use_quiescence: bool,
    quiescence_depth: i32,
}

impl UCIOptions {
    fn new() -> Self {
        UCIOptions { depth: 8, use_quiescence: true, quiescence_depth: 4 }
    }
}

struct SearchStats {
    nodes: i64,
    qnodes: i64,
    current_depth: i32,
    start_time: Instant,
}

impl SearchStats {
    fn new() -> Self {
        SearchStats { nodes: 0, qnodes: 0, current_depth: 0, start_time: Instant::now() }
    }

    fn nps(&self) -> i64 {
        let elapsed = Instant::now() - self.start_time;
        let ms = elapsed.as_millis();
        if ms == 0 { 0 } else { (self.nodes + self.qnodes) * 1000 / ms as i64 }
    }
}

// Forward declarations
struct Board {
    pieces: [[U64; 6]; 2],
    occupied: [U64; 2],
    all: U64,
    side: usize,
    ep: i32,
    castle: i32,
    hash: U64,
}

static mut KING_MOVES: [U64; 64] = [0; 64];
static mut KNIGHT_MOVES: [U64; 64] = [0; 64];
static mut ZOBRIST_PIECES: [[[U64; 64]; 2]; 6] = [[[0; 64]; 2]; 6];
static mut ZOBRIST_CASTLE: [U64; 16] = [0; 16];
static mut ZOBRIST_EP: [U64; 64] = [0; 64];
static mut ZOBRIST_SIDE: U64 = 0;

fn init_zobrist() {
    let mut rng = rand::thread_rng();
    for c in 0..2 {
        for p in 0..6 {
            for sq in 0..64 {
                unsafe {
                    ZOBRIST_PIECES[c][p][sq] = (rng.gen::<u64>() << 48) | (rng.gen::<u64>() << 32) |
                                               (rng.gen::<u64>() << 16) | rng.gen::<u64>();
                }
            }
        }
    }
    for i in 0..16 {
        unsafe {
            ZOBRIST_CASTLE[i] = (rng.gen::<u64>() << 48) | (rng.gen::<u64>() << 32) |
                                 (rng.gen::<u64>() << 16) | rng.gen::<u64>();
        }
    }
    for i in 0..64 {
        unsafe {
            ZOBRIST_EP[i] = (rng.gen::<u64>() << 48) | (rng.gen::<u64>() << 32) |
                             (rng.gen::<u64>() << 16) | rng.gen::<u64>();
        }
    }
    unsafe {
        ZOBRIST_SIDE = (rng.gen::<u64>() << 48) | (rng.gen::<u64>() << 32) |
                       (rng.gen::<u64>() << 16) | rng.gen::<u64>();
    }
}

fn zobrist_hash(b: &Board) -> U64 {
    let mut hash = 0;

    for c in 0..2 {
        for p in 0..6 {
            let mut bb = b.pieces[c][p];
            while bb != 0 {
                let sq = (bb as u64).trailing_zeros();
                hash ^= unsafe { ZOBRIST_PIECES[c][p][sq as usize] };
                bb &= bb - 1;
            }
        }
    }

    hash ^= unsafe { ZOBRIST_CASTLE[b.castle as usize] };
    if b.ep != -1 { hash ^= unsafe { ZOBRIST_EP[b.ep as usize] }; }
    if b.side == BLACK { hash ^= unsafe { ZOBRIST_SIDE }; }

    hash
}

fn is_attacked(sq: i32, attacker: usize, b: &Board) -> bool {
    if attacker == WHITE {
        if sq >= 9 && sq % 8 != 0 && (1u64 << (sq - 9)) & b.pieces[WHITE][PAWN] != 0 { return true; }
        if sq >= 7 && sq % 8 != 7 && (1u64 << (sq - 7)) & b.pieces[WHITE][PAWN] != 0 { return true; }
    } else {
        if sq <= 56 && sq % 8 != 0 && (1u64 << (sq + 7)) & b.pieces[BLACK][PAWN] != 0 { return true; }
        if sq <= 54 && sq % 8 != 7 && (1u64 << (sq + 9)) & b.pieces[BLACK][PAWN] != 0 { return true; }
    }

    if unsafe { KNIGHT_MOVES[sq as usize] } & b.pieces[attacker][KNIGHT] != 0 { return true; }
    if unsafe { KING_MOVES[sq as usize] } & b.pieces[attacker][KING] != 0 { return true; }
    if get_rook_attacks(sq, b.all) & (b.pieces[attacker][ROOK] | b.pieces[attacker][QUEEN]) != 0 { return true; }
    if get_bishop_attacks(sq, b.all) & (b.pieces[attacker][BISHOP] | b.pieces[attacker][QUEEN]) != 0 { return true; }

    false
}

fn get_rook_attacks(sq: i32, blockers: U64) -> U64 {
    let mut attacks = 0;
    let tr = sq / 8;
    let tf = sq % 8;

    for r in (tr + 1)..=7 {
        attacks |= 1u64 << (r * 8 + tf);
        if (1u64 << (r * 8 + tf)) & blockers != 0 { break; }
    }
    for r in (0..tr).rev() {
        attacks |= 1u64 << (r * 8 + tf);
        if (1u64 << (r * 8 + tf)) & blockers != 0 { break; }
    }
    for f in (tf + 1)..=7 {
        attacks |= 1u64 << (tr * 8 + f);
        if (1u64 << (tr * 8 + f)) & blockers != 0 { break; }
    }
    for f in (0..tf).rev() {
        attacks |= 1u64 << (tr * 8 + f);
        if (1u64 << (tr * 8 + f)) & blockers != 0 { break; }
    }
    attacks
}

fn get_bishop_attacks(sq: i32, blockers: U64) -> U64 {
    let mut attacks = 0;
    let tr = sq / 8;
    let tf = sq % 8;

    for (r, f) in (tr + 1..=7).zip(tf + 1..=7) {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & blockers != 0 { break; }
    }
    for (r, f) in (tr + 1..=7).zip((tf as i32 - 1)..=-1) {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & blockers != 0 { break; }
    }
    for (r, f) in (0..tr).rev().zip(tf + 1..=7) {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & blockers != 0 { break; }
    }
    for (r, f) in (0..tr).rev().zip(0..tf).rev() {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & blockers != 0 { break; }
    }
    attacks
}

fn init_tables() {
    for sq in 0..64 {
        let x = sq % 8;
        let y = sq / 8;

        unsafe {
            KING_MOVES[sq] = 0;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = (x as i32 + dx) as u32;
                    let ny = (y as i32 + dy) as u32;
                    if nx < 8 && ny < 8 {
                        KING_MOVES[sq] |= 1u64 << (ny * 8 + nx);
                    }
                }
            }

            KNIGHT_MOVES[sq] = 0;
            let kdx = [2, 2, -2, -2, 1, 1, -1, -1];
            let kdy = [1, -1, 1, -1, 2, -2, 2, -2];
            for i in 0..8 {
                let nx = (x as i32 + kdx[i]) as u32;
                let ny = (y as i32 + kdy[i]) as u32;
                if nx < 8 && ny < 8 {
                    KNIGHT_MOVES[sq] |= 1u64 << (ny * 8 + nx);
                }
            }
        }
    }

    init_zobrist();

    let _history_table = HistoryTable::new();
    let _killer_moves = KillerMoves::new();
    unsafe {
        std::ptr::write_bytes(TRANS_TABLE.as_mut_ptr(), 0, TRANS_TABLE.len());
    }
}

fn parse_move(b: &Board, move_str: &str) -> Option<Move> {
    let to = get_square(move_str.chars().nth(0).unwrap(), move_str.chars().nth(1).unwrap());
    let from = get_square(move_str.chars().nth(2).unwrap(), move_str.chars().nth(3).unwrap());
    let promo = if move_str.len() == 5 {
        match move_str.chars().nth(4).unwrap() {
            'q' => QUEEN as i32,
            'r' => ROOK as i32,
            'b' => BISHOP as i32,
            'n' => KNIGHT as i32,
            _ => -1,
        }
    } else {
        -1
    };

    let moves: Vec<Move> = Vec::new(); // Assuming this is a placeholder for actual move generation
    for m in moves {
        if m.from == from && m.to == to {
            if m.promo != -1 {
                if m.promo == promo { return Some(m); }
            } else {
                return Some(m);
            }
        }
    }
    None
}

fn get_square(file: char, rank: char) -> i32 {
    let f = (file as u8 - b'a') as i32;
    let r = (rank as u8 - b'1') as i32;
    if f < 0 || f > 7 || r < 0 || r > 7 {
        return -1;
    }
    r * 8 + f
}

fn main() {
    init_tables();
    let board = Board {
        pieces: [[0; 6]; 2],
        occupied: [0; 2],
        all: 0,
        side: WHITE,
        ep: -1,
        castle: 15,
        hash: 0,
    };

    // Assuming implementation for an init function for Board
    // board.init();

    let stdin = io::stdin();
    let reader = stdin.lock();
    let uci_options = UCIOptions::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let cmd = line.split_whitespace().next().unwrap();

        match cmd {
            "uci" => {
                println!("id name NanoChessTurbo");
                println!("id author CrvProject");
                println!("option name Depth type spin default 10 min 1 max 30");
                println!("option name Hash type spin default 64 min 1 max 1024");
                println!("uciok");
            }
            "setoption" => {
                let mut parts = line.split_whitespace();
                let _ = parts.next(); // "name"
                let option_name = parts.next().unwrap();
                if option_name == "Depth" {
                    let _value: i32 = parts.next().unwrap().parse().unwrap();
                    // uci_options.depth = value.max(1).min(30);
                }
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                // board.init();
            }
            "position" => {
                let mut parts = line.split_whitespace();
                let _ = parts.next(); // "position"
                let sub_cmd = parts.next().unwrap();
                if sub_cmd == "startpos" {
                    // board.init();
                } else if sub_cmd == "fen" {
                    // board.init();
                    while parts.next() != Some("moves") {}
                }

                if let Some(token) = parts.next() {
                    if token == "moves" {
                        while let Some(move_token) = parts.next() {
                            if let Some(_m) = parse_move(&board, move_token) {
                                // make_move(&mut board, m);
                            }
                        }
                    }
                }
            }
            "go" => {
                let mut _search_depth = uci_options.depth;
                let mut _move_time = 0;

                let mut parts = line.split_whitespace();
                while let Some(token) = parts.next() {
                    match token {
                        "depth" => {
                            _search_depth = parts.next().unwrap().parse().unwrap();
                        }
                        "movetime" => {
                            _move_time = parts.next().unwrap().parse().unwrap();
                        }
                        _ => {}
                    }
                }

                // let best_move = iterative_deepening(&mut board, search_depth);
                // if best_move.from != best_move.to {
                //     let move_str = format!("{:x}{:x}{:x}{:x}", best_move.from % 8, best_move.from / 8, best_move.to % 8, best_move.to / 8);
                //     println!("bestmove {}", move_str);
                // }
            }
            "quit" => break,
            _ => {},
        }
    }
}