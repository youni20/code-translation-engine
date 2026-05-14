use std::cmp::{max, min};
use std::io::{self, BufRead};
use std::time::{Duration, Instant};

type U64 = u64;

const PAWN: usize = 0;
const KNIGHT: usize = 1;
const BISHOP: usize = 2;
const ROOK: usize = 3;
const QUEEN: usize = 4;
const KING: usize = 5;

const WHITE: usize = 0;
const BLACK: usize = 1;

const INF: i32 = 999_999;
const MATE: i32 = 100_000;
const MAX_QUIESCENCE_DEPTH: i32 = 6;
const MAX_PLY: usize = 128;

const TT_SIZE: usize = 1 << 20;

const TT_EXACT: i32 = 0;
const TT_ALPHA: i32 = 1;
const TT_BETA: i32 = 2;

static mut KING_MOVES: Option<[U64; 64]> = None;
static mut KNIGHT_MOVES: Option<[U64; 64]> = None;

static mut ZOBRIST_PIECES: Option<[[[U64; 64]; 6]; 2]> = None;
static mut ZOBRIST_CASTLE: Option<[U64; 16]> = None;
static mut ZOBRIST_EP: Option<[U64; 64]> = None;
static mut ZOBRIST_SIDE: U64 = 0;

#[derive(Clone, Copy)]
struct HistoryTable {
    scores: [[[i32; 64]; 64]; 2],
}
impl HistoryTable {
    fn new() -> Self {
        HistoryTable {
            scores: [[[0; 64]; 64]; 2],
        }
    }
    fn init(&mut self) {
        for s in 0..2 {
            for f in 0..64 {
                for t in 0..64 {
                    self.scores[s][f][t] = 0;
                }
            }
        }
    }
    fn update(&mut self, side: usize, from: usize, to: usize, depth: i32) {
        let val = &mut self.scores[side][from][to];
        *val += depth * depth;
        if *val > 100_000 {
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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct Move {
    from: usize,
    to: usize,
    score: i32,
    piece: usize,
    captured: i32,
    promo: usize,
}
impl Move {
    fn new(f: usize, t: usize, p: usize, c: i32, pr: usize) -> Self {
        Move {
            from: f,
            to: t,
            piece: p,
            captured: c,
            promo: pr,
            score: 0,
        }
    }
}

struct KillerMoves {
    killers: Vec<[Option<Move>; 2]>, // indexed by ply
}
impl KillerMoves {
    fn new() -> Self {
        KillerMoves {
            killers: vec![[None, None]; MAX_PLY],
        }
    }
    fn init(&mut self) {
        for i in 0..self.killers.len() {
            self.killers[i] = [None, None];
        }
    }
    fn update(&mut self, m: Move, ply: usize) {
        if self.killers[ply][0] != Some(m) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(m);
        }
    }
    fn is_killer(&self, m: &Move, ply: usize) -> bool {
        self.killers[ply][0] == Some(*m) || self.killers[ply][1] == Some(*m)
    }
}

#[derive(Clone, Copy, Default)]
struct TTEntry {
    hash: U64,
    depth: i32,
    score: i32,
    flag: i32,
    best_move: u32,
}

struct UCIOptions {
    depth: i32,
    use_quiescence: bool,
    quiescence_depth: i32,
}
impl Default for UCIOptions {
    fn default() -> Self {
        UCIOptions {
            depth: 8,
            use_quiescence: true,
            quiescence_depth: 4,
        }
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
        SearchStats {
            nodes: 0,
            qnodes: 0,
            current_depth: 0,
            start_time: Instant::now(),
        }
    }
    fn init(&mut self) {
        self.nodes = 0;
        self.qnodes = 0;
        self.current_depth = 0;
        self.start_time = Instant::now();
    }
    fn nps(&self) -> i64 {
        let elapsed = self.start_time.elapsed();
        let ms = elapsed.as_millis() as i64;
        if ms == 0 {
            return 0;
        }
        (self.nodes + self.qnodes) * 1000 / ms
    }
}

#[derive(Clone)]
struct Board {
    pieces: [[U64; 6]; 2],
    occupied: [U64; 2],
    all: U64,
    side: usize,
    ep: i32,
    castle: usize,
    hash: U64,
}
impl Board {
    fn new() -> Self {
        Board {
            pieces: [[0; 6]; 2],
            occupied: [0; 2],
            all: 0,
            side: WHITE,
            ep: -1,
            castle: 15,
            hash: 0,
        }
    }
    fn init(&mut self) {
        for c in 0..2 {
            for p in 0..6 {
                self.pieces[c][p] = 0;
            }
        }
        self.pieces[WHITE][PAWN] = 0xFF00u64;
        self.pieces[WHITE][KNIGHT] = 0x42u64;
        self.pieces[WHITE][BISHOP] = 0x24u64;
        self.pieces[WHITE][ROOK] = 0x81u64;
        self.pieces[WHITE][QUEEN] = 0x8u64;
        self.pieces[WHITE][KING] = 0x10u64;

        self.pieces[BLACK][PAWN] = 0xFF000000000000u64;
        self.pieces[BLACK][KNIGHT] = 0x4200000000000000u64;
        self.pieces[BLACK][BISHOP] = 0x2400000000000000u64;
        self.pieces[BLACK][ROOK] = 0x8100000000000000u64;
        self.pieces[BLACK][QUEEN] = 0x800000000000000u64;
        self.pieces[BLACK][KING] = 0x1000000000000000u64;

        self.update();
        self.side = WHITE;
        self.ep = -1;
        self.castle = 15;
        self.hash = zobrist_hash(self);
    }
    fn update(&mut self) {
        self.occupied[WHITE] = 0;
        self.occupied[BLACK] = 0;
        for p in PAWN..=KING {
            self.occupied[WHITE] |= self.pieces[WHITE][p];
            self.occupied[BLACK] |= self.pieces[BLACK][p];
        }
        self.all = self.occupied[WHITE] | self.occupied[BLACK];
    }
    fn evaluate(&self) -> i32 {
        let mut eval: i32 = 0;
        let values = [100, 320, 330, 500, 900, 0];

        for c in 0..2 {
            for p in 0..6 {
                let count = self.pieces[c][p].count_ones() as i32;
                if c == WHITE {
                    eval += count * values[p];
                } else {
                    eval -= count * values[p];
                }
            }
        }

        for c in 0..2 {
            if self.pieces[c][KING] == 0 {
                continue;
            }
            let king_sq = self.pieces[c][KING].trailing_zeros() as usize;
            if c == WHITE {
                if king_sq == 6 || king_sq == 2 {
                    eval += 40;
                } else if king_sq == 4 {
                    eval -= 20;
                }
            } else {
                if king_sq == 62 || king_sq == 58 {
                    eval -= 40;
                } else if king_sq == 60 {
                    eval += 20;
                }
            }
        }

        let center: U64 = 0x0000001818000000u64;
        eval += (self.pieces[WHITE][PAWN] & center).count_ones() as i32 * 20;
        eval -= (self.pieces[BLACK][PAWN] & center).count_ones() as i32 * 20;

        let mut wpawns = self.pieces[WHITE][PAWN];
        while wpawns != 0 {
            let sq = wpawns.trailing_zeros() as usize;
            let rank = (sq / 8) as i32;
            if rank >= 4 {
                eval += (rank - 3) * 15;
            }
            wpawns &= wpawns - 1;
        }

        let mut bpawns = self.pieces[BLACK][PAWN];
        while bpawns != 0 {
            let sq = bpawns.trailing_zeros() as usize;
            let rank = (sq / 8) as i32;
            if rank <= 3 {
                eval -= (4 - rank) * 15;
            }
            bpawns &= bpawns - 1;
        }

        if self.side == WHITE {
            eval
        } else {
            -eval
        }
    }
}

fn zobrist_hash(b: &Board) -> U64 {
    let mut hash: U64 = 0;
    unsafe {
        let pieces = ZOBRIST_PIECES.as_ref().unwrap();
        for c in 0..2 {
            for p in 0..6 {
                let mut bb = b.pieces[c][p];
                while bb != 0 {
                    let sq = bb.trailing_zeros() as usize;
                    hash ^= pieces[c][p][sq];
                    bb &= bb - 1;
                }
            }
        }
        let castle = ZOBRIST_CASTLE.as_ref().unwrap();
        hash ^= castle[b.castle];
        if b.ep != -1 {
            let ep = ZOBRIST_EP.as_ref().unwrap();
            hash ^= ep[b.ep as usize];
        }
        if b.side == BLACK {
            hash ^= ZOBRIST_SIDE;
        }
    }
    hash
}

fn get_rook_attacks(sq: usize, blockers: U64) -> U64 {
    let mut attacks: U64 = 0;
    let tr = sq / 8;
    let tf = sq % 8;

    for r in (tr + 1)..=7 {
        let sq2 = r * 8 + tf;
        attacks |= 1u64 << sq2;
        if (1u64 << sq2) & blockers != 0 {
            break;
        }
    }
    if tr >= 1 {
        for r in (0..tr).rev() {
            let sq2 = r * 8 + tf;
            attacks |= 1u64 << sq2;
            if (1u64 << sq2) & blockers != 0 {
                break;
            }
        }
    }
    for f in (tf + 1)..=7 {
        let sq2 = tr * 8 + f;
        attacks |= 1u64 << sq2;
        if (1u64 << sq2) & blockers != 0 {
            break;
        }
    }
    if tf >= 1 {
        for f in (0..tf).rev() {
            let sq2 = tr * 8 + f;
            attacks |= 1u64 << sq2;
            if (1u64 << sq2) & blockers != 0 {
                break;
            }
        }
    }
    attacks
}

fn get_bishop_attacks(sq: usize, blockers: U64) -> U64 {
    let mut attacks: U64 = 0;
    let tr = sq / 8;
    let tf = sq % 8;

    let mut r = tr as i32 + 1;
    let mut f = tf as i32 + 1;
    while r <= 7 && f <= 7 {
        let sq2 = (r as usize) * 8 + (f as usize);
        attacks |= 1u64 << sq2;
        if (1u64 << sq2) & blockers != 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    let mut r = tr as i32 + 1;
    let mut f = tf as i32 - 1;
    while r <= 7 && f >= 0 {
        let sq2 = (r as usize) * 8 + (f as usize);
        attacks |= 1u64 << sq2;
        if (1u64 << sq2) & blockers != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    let mut r = tr as i32 - 1;
    let mut f = tf as i32 + 1;
    while r >= 0 && f <= 7 {
        let sq2 = (r as usize) * 8 + (f as usize);
        attacks |= 1u64 << sq2;
        if (1u64 << sq2) & blockers != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    let mut r = tr as i32 - 1;
    let mut f = tf as i32 - 1;
    while r >= 0 && f >= 0 {
        let sq2 = (r as usize) * 8 + (f as usize);
        attacks |= 1u64 << sq2;
        if (1u64 << sq2) & blockers != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    attacks
}

fn is_attacked(sq: usize, attacker: usize, b: &Board) -> bool {
    unsafe {
        let km = KING_MOVES.as_ref().unwrap();
        let nm = KNIGHT_MOVES.as_ref().unwrap();
        if attacker == WHITE {
            if sq >= 9 && (sq % 8) != 0 && ((1u64 << (sq - 9)) & b.pieces[WHITE][PAWN]) != 0 {
                return true;
            }
            if sq >= 7 && (sq % 8) != 7 && ((1u64 << (sq - 7)) & b.pieces[WHITE][PAWN]) != 0 {
                return true;
            }
        } else {
            if sq <= 56 && (sq % 8) != 0 && ((1u64 << (sq + 7)) & b.pieces[BLACK][PAWN]) != 0 {
                return true;
            }
            if sq <= 54 && (sq % 8) != 7 && ((1u64 << (sq + 9)) & b.pieces[BLACK][PAWN]) != 0 {
                return true;
            }
        }

        if (nm[sq] & b.pieces[attacker][KNIGHT]) != 0 {
            return true;
        }
        if (km[sq] & b.pieces[attacker][KING]) != 0 {
            return true;
        }
        if (get_rook_attacks(sq, b.all) & (b.pieces[attacker][ROOK] | b.pieces[attacker][QUEEN])) != 0 {
            return true;
        }
        if (get_bishop_attacks(sq, b.all) & (b.pieces[attacker][BISHOP] | b.pieces[attacker][QUEEN])) != 0 {
            return true;
        }
    }
    false
}

fn is_in_check(b: &Board) -> bool {
    if b.pieces[b.side][KING] == 0 {
        return false;
    }
    let king_sq = b.pieces[b.side][KING].trailing_zeros() as usize;
    is_attacked(king_sq, 1 - b.side, b)
}

fn make_move(b: &mut Board, m: &Move) {
    let from_bb: U64 = 1u64 << m.from;
    let to_bb: U64 = 1u64 << m.to;
    let opponent = 1 - b.side;
    let prev_ep = b.ep;
    unsafe {
        let pieces_z = ZOBRIST_PIECES.as_ref().unwrap();
        let ep_z = ZOBRIST_EP.as_ref().unwrap();
        let castle_z = ZOBRIST_CASTLE.as_ref().unwrap();

        b.hash ^= pieces_z[b.side][m.piece][m.from];
        b.hash ^= pieces_z[b.side][m.piece][m.to];

        if b.ep != -1 {
            b.hash ^= ep_z[b.ep as usize];
        }
        b.hash ^= castle_z[b.castle];

        if m.piece == KING {
            if b.side == WHITE {
                b.castle &= !3;
            } else {
                b.castle &= !12;
            }
        }
        if m.from == 0 || m.to == 0 {
            b.castle &= !2;
        }
        if m.from == 7 || m.to == 7 {
            b.castle &= !1;
        }
        if m.from == 56 || m.to == 56 {
            b.castle &= !8;
        }
        if m.from == 63 || m.to == 63 {
            b.castle &= !4;
        }

        b.hash ^= castle_z[b.castle];
        b.ep = -1;

        b.pieces[b.side][m.piece] ^= from_bb | to_bb;

        for p in PAWN..=KING {
            if (b.pieces[opponent][p] & to_bb) != 0 {
                b.pieces[opponent][p] ^= to_bb;
                b.hash ^= pieces_z[opponent][p][m.to];
                break;
            }
        }

        if m.piece == PAWN {
            if m.to == prev_ep as usize && prev_ep != -1 {
                let captured_pawn_sq = if b.side == WHITE {
                    m.to - 8
                } else {
                    m.to + 8
                };
                b.pieces[opponent][PAWN] ^= 1u64 << captured_pawn_sq;
                b.hash ^= pieces_z[opponent][PAWN][captured_pawn_sq];
            }
            if (if b.side == WHITE { (m.from as i32 - m.to as i32).abs() } else { (m.to as i32 - m.from as i32).abs() }) == 16 {
                b.ep = if b.side == WHITE { (m.from + 8) as i32 } else { (m.from as i32 - 8) as i32 };
                if b.ep >= 0 {
                    b.hash ^= ep_z[b.ep as usize];
                }
            }
            if m.promo != 0 {
                b.pieces[b.side][PAWN] ^= to_bb;
                b.pieces[b.side][m.promo] ^= to_bb;
                b.hash ^= pieces_z[b.side][PAWN][m.to];
                b.hash ^= pieces_z[b.side][m.promo][m.to];
            }
        } else if m.piece == KING {
            if (m.from as i32 - m.to as i32).abs() == 2 {
                if m.to == 6 {
                    b.pieces[WHITE][ROOK] ^= (1u64 << 7) | (1u64 << 5);
                    b.hash ^= pieces_z[WHITE][ROOK][7];
                    b.hash ^= pieces_z[WHITE][ROOK][5];
                } else if m.to == 2 {
                    b.pieces[WHITE][ROOK] ^= (1u64 << 0) | (1u64 << 3);
                    b.hash ^= pieces_z[WHITE][ROOK][0];
                    b.hash ^= pieces_z[WHITE][ROOK][3];
                } else if m.to == 62 {
                    b.pieces[BLACK][ROOK] ^= (1u64 << 63) | (1u64 << 61);
                    b.hash ^= pieces_z[BLACK][ROOK][63];
                    b.hash ^= pieces_z[BLACK][ROOK][61];
                } else if m.to == 58 {
                    b.pieces[BLACK][ROOK] ^= (1u64 << 56) | (1u64 << 59);
                    b.hash ^= pieces_z[BLACK][ROOK][56];
                    b.hash ^= pieces_z[BLACK][ROOK][59];
                }
            }
        }

        b.update();
        b.side = opponent;
        b.hash ^= ZOBRIST_SIDE;
    }
}

fn is_legal_move(b: &Board, m: &Move) -> bool {
    let mut copy = b.clone();
    make_move(&mut copy, m);
    if copy.pieces[b.side][KING] == 0 {
        return false;
    }
    let king_sq = copy.pieces[b.side][KING].trailing_zeros() as usize;
    !is_attacked(king_sq, copy.side, &copy)
}

fn generate_moves(b: &Board, captures_only: bool) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    moves.reserve(if captures_only { 32 } else { 128 });

    let mut bitboard: U64;
    let mut attacks: U64;
    let mut from: usize;
    let mut to: usize;

    for p in PAWN..=KING {
        bitboard = b.pieces[b.side][p];
        while bitboard != 0 {
            from = bitboard.trailing_zeros() as usize;
            attacks = 0;

            if p == PAWN {
                let dir: i32 = if b.side == WHITE { 8 } else { -8 };
                let promo_rank: i32 = if b.side == WHITE { 7 } else { 0 };

                if !captures_only {
                    let to_sq = (from as i32 + dir) as i32;
                    if to_sq >= 0 && to_sq < 64 && (b.all & (1u64 << to_sq as usize)) == 0 {
                        if (to_sq / 8) == promo_rank {
                            moves.push(Move::new(from, to_sq as usize, p, -1, QUEEN));
                        } else {
                            moves.push(Move::new(from, to_sq as usize, p, -1, 0));
                            let start_rank = if b.side == WHITE { 1 } else { 6 };
                            if (from / 8) == start_rank {
                                let to_sq2 = from as i32 + 2 * dir;
                                if to_sq2 >= 0 && to_sq2 < 64 && (b.all & (1u64 << to_sq2 as usize)) == 0 {
                                    moves.push(Move::new(from, to_sq2 as usize, p, -1, 0));
                                }
                            }
                        }
                    }
                }

                let cap_dirs = [dir - 1, dir + 1];
                for &d in cap_dirs.iter() {
                    let to_i = from as i32 + d;
                    if to_i < 0 || to_i > 63 {
                        continue;
                    }
                    let tou = to_i as usize;
                    if ((from % 8) as i32 - (tou as i32 % 8)) .abs() > 1 {
                        // skip invalid wrap
                        if (from % 8).abs_diff(tou % 8) > 1 {
                            continue;
                        }
                    }
                    if (b.occupied[1 - b.side] & (1u64 << tou)) != 0 {
                        if (tou / 8) as i32 == promo_rank {
                            moves.push(Move::new(from, tou, p, 0, QUEEN));
                        } else {
                            moves.push(Move::new(from, tou, p, -1, 0));
                        }
                    } else if !captures_only && (b.ep == tou as i32) {
                        moves.push(Move::new(from, tou, p, -1, 0));
                    }
                }
            } else if p == KING && !captures_only {
                unsafe {
                    let km = KING_MOVES.as_ref().unwrap();
                    attacks = km[from] & !b.occupied[b.side];
                }
                if !is_in_check(b) {
                    if b.side == WHITE {
                        if (b.castle & 1) != 0 && (b.all & 0x60u64) == 0 {
                            if !is_attacked(5, BLACK, b) && !is_attacked(6, BLACK, b) {
                                moves.push(Move::new(4, 6, KING, -1, 0));
                            }
                        }
                        if (b.castle & 2) != 0 && (b.all & 0xEu64) == 0 {
                            if !is_attacked(3, BLACK, b) && !is_attacked(2, BLACK, b) {
                                moves.push(Move::new(4, 2, KING, -1, 0));
                            }
                        }
                    } else {
                        if (b.castle & 4) != 0 && (b.all & 0x6000000000000000u64) == 0 {
                            if !is_attacked(61, WHITE, b) && !is_attacked(62, WHITE, b) {
                                moves.push(Move::new(60, 62, KING, -1, 0));
                            }
                        }
                        if (b.castle & 8) != 0 && (b.all & 0xE00000000000000u64) == 0 {
                            if !is_attacked(59, WHITE, b) && !is_attacked(58, WHITE, b) {
                                moves.push(Move::new(60, 58, KING, -1, 0));
                            }
                        }
                    }
                }
            } else {
                if p == KNIGHT {
                    unsafe {
                        let nm = KNIGHT_MOVES.as_ref().unwrap();
                        attacks = nm[from];
                    }
                } else if p == BISHOP {
                    attacks = get_bishop_attacks(from, b.all);
                } else if p == ROOK {
                    attacks = get_rook_attacks(from, b.all);
                } else if p == QUEEN {
                    attacks = get_rook_attacks(from, b.all) | get_bishop_attacks(from, b.all);
                } else if p == KING {
                    unsafe {
                        let km = KING_MOVES.as_ref().unwrap();
                        attacks = km[from];
                    }
                }
                if captures_only {
                    attacks &= b.occupied[1 - b.side];
                } else {
                    attacks &= !b.occupied[b.side];
                }
            }

            while attacks != 0 {
                to = attacks.trailing_zeros() as usize;
                moves.push(Move::new(from, to, p, -1, 0));
                attacks &= attacks - 1;
            }

            bitboard &= bitboard - 1;
        }
    }

    let mut legal_moves: Vec<Move> = Vec::with_capacity(moves.len());
    for m in moves.iter() {
        if is_legal_move(b, m) {
            legal_moves.push(*m);
        }
    }
    legal_moves
}

unsafe fn get_history_table<'a>() -> &'a mut HistoryTable {
    static mut HISTORY_TABLE: Option<HistoryTable> = None;
    if HISTORY_TABLE.is_none() {
        HISTORY_TABLE = Some(HistoryTable::new());
    }
    HISTORY_TABLE.as_mut().unwrap()
}

unsafe fn get_killer_moves<'a>() -> &'a mut KillerMoves {
    static mut KILLER_MOVES: Option<KillerMoves> = None;
    if KILLER_MOVES.is_none() {
        KILLER_MOVES = Some(KillerMoves::new());
    }
    KILLER_MOVES.as_mut().unwrap()
}

unsafe fn get_transposition_table<'a>() -> &'a mut Vec<TTEntry> {
    static mut TRANSPOSITION_TABLE: Option<Vec<TTEntry>> = None;
    if TRANSPOSITION_TABLE.is_none() {
        TRANSPOSITION_TABLE = Some(vec![TTEntry::default(); TT_SIZE]);
    }
    TRANSPOSITION_TABLE.as_mut().unwrap()
}

fn score_moves(moves: &mut Vec<Move>, b: &Board, tt_move: Option<Move>, ply: usize) {
    unsafe {
        let history = get_history_table();
        let killer_moves = get_killer_moves();
        for m in moves.iter_mut() {
            if let Some(ttm) = tt_move {
                if *m == ttm {
                    m.score = 1_000_000;
                    continue;
                }
            }
            if (b.occupied[1 - b.side] & (1u64 << m.to)) != 0 {
                let victim_values = [100, 300, 300, 500, 900, 10_000];
                for p in (PAWN..=KING).rev() {
                    if (b.pieces[1 - b.side][p] & (1u64 << m.to)) != 0 {
                        m.score = 100_000 + victim_values[p] * 10 - victim_values[m.piece];
                        break;
                    }
                }
            } else if killer_moves.is_killer(m, ply) {
                m.score = 90_000;
            } else {
                m.score = history.get(b.side, m.from, m.to);
            }
            if m.promo == QUEEN {
                m.score += 80_000;
            }
        }
        moves.sort_by(|a, b| b.score.cmp(&a.score));
    }
}

static mut UCI_OPTIONS: UCIOptions = UCIOptions {
    depth: 8,
    use_quiescence: true,
    quiescence_depth: 4,
};

static mut SEARCH_STATS: Option<SearchStats> = None;

fn quiescence(b: &Board, mut alpha: i32, beta: i32, depth: i32) -> i32 {
    unsafe {
        if SEARCH_STATS.is_none() {
            SEARCH_STATS = Some(SearchStats::new());
        }
        let stats = SEARCH_STATS.as_mut().unwrap();
        stats.qnodes += 1;
    }

    let stand_pat = b.evaluate();

    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }
    if depth <= -MAX_QUIESCENCE_DEPTH {
        return stand_pat;
    }

    let mut captures = generate_moves(b, true);
    score_moves(&mut captures, b, None, 0);

    for m in captures.iter() {
        let mut gain = 200;
        if m.piece != PAWN {
            gain = 900;
        }
        if stand_pat + gain < alpha && depth < -1 {
            continue;
        }
        let mut copy = b.clone();
        make_move(&mut copy, m);
        let score = -quiescence(&copy, -beta, -alpha, depth - 1);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn search(
    b: &Board,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    best_move: &mut Move,
    ply: usize,
    null_move: bool,
) -> i32 {
    unsafe {
        if SEARCH_STATS.is_none() {
            SEARCH_STATS = Some(SearchStats::new());
        }
        let stats = SEARCH_STATS.as_mut().unwrap();
        stats.nodes += 1;
    }

    let mut in_check = is_in_check(b);
    let mut depth = depth;
    if in_check {
        depth += 1;
    }

    unsafe {
        let tt = get_transposition_table();
        let tt_index = (b.hash as usize) % TT_SIZE;
        let tt_entry = tt.get(tt_index).unwrap().clone();
        let mut tt_move_opt: Option<Move> = None;

        if tt_entry.hash == b.hash && tt_entry.depth >= depth {
            if tt_entry.flag == TT_EXACT {
                if ply == 0 {
                    best_move.from = (tt_entry.best_move & 63) as usize;
                    best_move.to = ((tt_entry.best_move >> 6) & 63) as usize;
                    best_move.piece = ((tt_entry.best_move >> 12) & 7) as usize;
                }
                return tt_entry.score;
            }
            if tt_entry.flag == TT_ALPHA && tt_entry.score <= alpha {
                return alpha;
            }
            if tt_entry.flag == TT_BETA && tt_entry.score >= beta {
                return beta;
            }
        }
        if tt_entry.hash == b.hash && tt_entry.best_move != 0 {
            let bm = tt_entry.best_move;
            tt_move_opt = Some(Move::new(
                (bm & 63) as usize,
                ((bm >> 6) & 63) as usize,
                ((bm >> 12) & 7) as usize,
                -1,
                0,
            ));
        }

        if depth <= 0 {
            return quiescence(b, alpha, beta, 0);
        }

        if null_move && !in_check && depth >= 3 && ply > 0 {
            let mut copy = b.clone();
            copy.side = 1 - copy.side;
            copy.hash ^= ZOBRIST_SIDE;
            copy.ep = -1;

            let mut dummy = Move::default();
            let r = if depth > 6 { 3 } else { 2 };
            let score = -search(&copy, depth - 1 - r, -beta, -beta + 1, &mut dummy, ply + 1, false);
            if score >= beta {
                return beta;
            }
        }

        let mut moves = generate_moves(b, false);
        if moves.is_empty() {
            if in_check {
                return -MATE + ply as i32;
            }
            return 0;
        }

        score_moves(&mut moves, b, tt_move_opt, ply);

        if ply == 0 && !moves.is_empty() {
            *best_move = moves[0];
        }

        let mut move_count = 0;
        let mut best_score = -INF;
        let mut local_best = Move::default();
        let orig_alpha = alpha;

        for m in moves.iter_mut() {
            move_count += 1;
            let mut reduction = 0;
            if move_count > 4 && depth >= 3 && !in_check && (b.occupied[1 - b.side] & (1u64 << m.to)) == 0 && m.promo == 0 {
                if move_count > 12 {
                    reduction = 3;
                } else if move_count > 6 {
                    reduction = 2;
                } else {
                    reduction = 1;
                }
                let history = get_history_table();
                let killer_moves = get_killer_moves();
                if killer_moves.is_killer(m, ply) || history.get(b.side, m.from, m.to) > 5000 {
                    reduction = std::cmp::max(0, reduction - 1);
                }
            }

            let mut copy = b.clone();
            make_move(&mut copy, m);

            let score: i32;
            if move_count == 1 {
                let mut dummy = Move::default();
                score = -search(&copy, depth - 1 - reduction, -beta, -alpha, &mut dummy, ply + 1, true);
            } else {
                let mut dummy = Move::default();
                let mut sc = -search(&copy, depth - 1 - reduction, -alpha - 1, -alpha, &mut dummy, ply + 1, true);
                if sc > alpha && sc < beta {
                    sc = -search(&copy, depth - 1, -beta, -alpha, &mut dummy, ply + 1, true);
                }
                // Reassign
                let score_re = sc;
                // Use score_re as score
                if reduction > 0 && score_re > alpha {
                    let mut dummy2 = Move::default();
                    let sc2 = -search(&copy, depth - 1, -beta, -alpha, &mut dummy2, ply + 1, true);
                    score = sc2;
                } else {
                    score = score_re;
                }
            }

            if score > best_score {
                best_score = score;
                local_best = *m;
                if ply == 0 {
                    *best_move = *m;
                }
            }

            if score > alpha {
                alpha = score;
                if (b.occupied[1 - b.side] & (1u64 << m.to)) == 0 {
                    let history = get_history_table();
                    history.update(b.side, m.from, m.to, depth);
                }
            }

            if alpha >= beta {
                if (b.occupied[1 - b.side] & (1u64 << m.to)) == 0 {
                    let killer_moves = get_killer_moves();
                    killer_moves.update(*m, ply);
                }
                break;
            }

            if depth <= 2 && !in_check && move_count > 8 && (b.occupied[1 - b.side] & (1u64 << m.to)) == 0 {
                let futility_margin = depth * 100;
                if b.evaluate() + futility_margin < alpha {
                    break;
                }
            }
        }

        let tt_mut = get_transposition_table();
        let entry = &mut tt_mut[tt_index];
        entry.hash = b.hash;
        entry.depth = depth;
        entry.score = best_score;
        entry.best_move = (local_best.from as u32) | ((local_best.to as u32) << 6) | ((local_best.piece as u32) << 12);
        if best_score <= orig_alpha {
            entry.flag = TT_ALPHA;
        } else if best_score >= beta {
            entry.flag = TT_BETA;
        } else {
            entry.flag = TT_EXACT;
        }

        best_score
    }
}

fn iterative_deepening(b: &Board, max_depth: i32, best_move: &mut Move, time_limit_ms: i32) -> i32 {
    let mut score = 0;
    let mut alpha = -INF;
    let mut beta = INF;
    let mut window = 50;

    unsafe {
        if SEARCH_STATS.is_none() {
            SEARCH_STATS = Some(SearchStats::new());
        }
        let stats = SEARCH_STATS.as_mut().unwrap();
        stats.init();
    }

    for depth in 1..=max_depth {
        unsafe {
            SEARCH_STATS.as_mut().unwrap().current_depth = depth;
        }

        if depth >= 4 {
            alpha = score - window;
            beta = score + window;
        }

        let temp_score = {
            let mut bm = Move::default();
            let s = search(b, depth, alpha, beta, &mut bm, 0, true);
            *best_move = bm;
            s
        };

        let mut final_score = temp_score;
        if temp_score <= alpha || temp_score >= beta {
            let mut bm = Move::default();
            final_score = search(b, depth, -INF, INF, &mut bm, 0, true);
            *best_move = bm;
            window = 50;
        } else {
            window = 25;
        }

        score = final_score;

        if time_limit_ms > 0 {
            let elapsed_ms = unsafe { SEARCH_STATS.as_ref().unwrap().start_time.elapsed().as_millis() as i64 };
            if (elapsed_ms as i32) > (time_limit_ms as i32) * 40 / 100 && depth > 4 {
                break;
            }
        }

        // Output UCI info
        print!("info depth {}", depth);
        print!(" score ");
        if score.abs() >= MATE - 1000 {
            let mut mate_in = (MATE - score.abs() + 1) / 2;
            if score < 0 {
                mate_in = -mate_in;
            }
            print!("mate {}", mate_in);
        } else {
            print!("cp {}", score);
        }
        unsafe {
            let stats = SEARCH_STATS.as_ref().unwrap();
            print!(" nodes {}", stats.nodes);
            print!(" nps {}", stats.nps());
        }
        print!(" pv ");

        let mut move_str = String::new();
        move_str.push((best_move.from % 8) as u8 as char);
        // Above is wrong for converting numeric to char; fix properly
        // Rebuild properly:
        move_str = String::new();
        move_str.push(((best_move.from % 8) as u8 + b'a') as char);
        move_str.push(((best_move.from / 8) as u8 + b'1') as char);
        move_str.push(((best_move.to % 8) as u8 + b'a') as char);
        move_str.push(((best_move.to / 8) as u8 + b'1') as char);
        if best_move.promo != 0 {
            match best_move.promo {
                QUEEN => move_str.push('q'),
                ROOK => move_str.push('r'),
                BISHOP => move_str.push('b'),
                KNIGHT => move_str.push('n'),
                _ => {}
            }
        }
        println!("{}{}", " ", move_str);

        if score.abs() >= MATE - 1000 {
            break;
        }
    }

    score
}

struct SimpleRng {
    state: u64,
}
impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        // simple LCG
        self.state = self.state.wrapping_mul(6364136223846793005u64).wrapping_add(1);
        self.state
    }
}

fn init_zobrist() {
    unsafe {
        let mut pieces = [[[0u64; 64]; 6]; 2];
        let mut castle = [0u64; 16];
        let mut ep = [0u64; 64];
        let mut rng = SimpleRng::new(12345);
        for c in 0..2 {
            for p in 0..6 {
                for sq in 0..64 {
                    let r = rng.next_u64();
                    pieces[c][p][sq] = r;
                }
            }
        }
        for i in 0..16 {
            castle[i] = rng.next_u64();
        }
        for i in 0..64 {
            ep[i] = rng.next_u64();
        }
        ZOBRIST_PIECES = Some(pieces);
        ZOBRIST_CASTLE = Some(castle);
        ZOBRIST_EP = Some(ep);
        ZOBRIST_SIDE = rng.next_u64();
    }
}

fn init_tables() {
    unsafe {
        let mut km = [0u64; 64];
        let mut nm = [0u64; 64];
        for sq in 0..64 {
            let x = sq % 8;
            let y = sq / 8;
            km[sq] = 0;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                        km[sq] |= 1u64 << (ny as usize * 8 + nx as usize);
                    }
                }
            }
            nm[sq] = 0;
            let kdx = [2, 2, -2, -2, 1, 1, -1, -1];
            let kdy = [1, -1, 1, -1, 2, -2, 2, -2];
            for i in 0..8 {
                let nx = x as i32 + kdx[i];
                let ny = y as i32 + kdy[i];
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    nm[sq] |= 1u64 << (ny as usize * 8 + nx as usize);
                }
            }
        }
        KING_MOVES = Some(km);
        KNIGHT_MOVES = Some(nm);
        init_zobrist();
        let history = get_history_table();
        history.init();
        let killer = get_killer_moves();
        killer.init();
        let tt = get_transposition_table();
        for e in tt.iter_mut() {
            *e = TTEntry::default();
        }
    }
}

fn parse_move(board: &Board, move_str: &str, parsed_move: &mut Move) -> bool {
    let moves = generate_moves(board, false);
    if move_str.len() < 4 {
        return false;
    }
    let from = (move_str.as_bytes()[0] - b'a') as usize + (move_str.as_bytes()[1] - b'1') as usize * 8;
    let to = (move_str.as_bytes()[2] - b'a') as usize + (move_str.as_bytes()[3] - b'1') as usize * 8;
    let mut promo_piece = 0usize;
    if move_str.len() == 5 {
        match move_str.as_bytes()[4] as char {
            'q' => promo_piece = QUEEN,
            'r' => promo_piece = ROOK,
            'b' => promo_piece = BISHOP,
            'n' => promo_piece = KNIGHT,
            _ => {}
        }
    }
    for m in moves.iter() {
        if m.from == from && m.to == to {
            if m.promo != 0 {
                if m.promo == promo_piece {
                    *parsed_move = *m;
                    return true;
                }
            } else {
                *parsed_move = *m;
                return true;
            }
        }
    }
    false
}

fn main() {
    init_tables();
    let mut board = Board::new();
    board.init();

    unsafe {
        UCI_OPTIONS = UCIOptions::default();
    }

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let cmd_opt = parts.next();
        if cmd_opt.is_none() {
            continue;
        }
        let cmd = cmd_opt.unwrap();
        if cmd == "uci" {
            println!("id name NanoChessTurbo");
            println!("id author CrvProject");
            println!("option name Depth type spin default 10 min 1 max 30");
            println!("option name Hash type spin default 64 min 1 max 1024");
            println!("uciok");
        } else if cmd == "setoption" {
            let mut token = parts.next();
            if token == Some("name") {
                let mut option_name = String::new();
                while let Some(tok) = parts.next() {
                    if tok == "value" {
                        break;
                    }
                    if !option_name.is_empty() {
                        option_name.push(' ');
                    }
                    option_name.push_str(tok);
                }
                if option_name == "Depth" {
                    if let Some(val_str) = parts.next() {
                        if let Ok(value) = val_str.parse::<i32>() {
                            unsafe {
                                UCI_OPTIONS.depth = max(1, min(30, value));
                            }
                        }
                    }
                } else if option_name == "Hash" {
                    if let Some(_val_str) = parts.next() {
                        // Ignored: could resize TT here
                    }
                }
            }
        } else if cmd == "isready" {
            println!("readyok");
        } else if cmd == "ucinewgame" {
            board.init();
            unsafe {
                let history = get_history_table();
                history.init();
                let killer = get_killer_moves();
                killer.init();
                let tt = get_transposition_table();
                for e in tt.iter_mut() {
                    *e = TTEntry::default();
                }
            }
        } else if cmd == "position" {
            let sub_cmd = parts.next();
            let mut token_after = None;
            if let Some(sub) = sub_cmd {
                if sub == "startpos" {
                    board.init();
                    token_after = parts.next();
                } else if sub == "fen" {
                    board.init();
                    // consume fen fields until "moves" or end
                    while let Some(tok) = parts.next() {
                        if tok == "moves" {
                            token_after = Some("moves");
                            break;
                        }
                    }
                } else {
                    // ignore
                }
            }
            if token_after == Some("moves") || (sub_cmd == Some("startpos") && token_after.is_some()) {
                // parse moves
                let mut mv = Move::default();
                for tok in parts {
                    if parse_move(&board, tok, &mut mv) {
                        make_move(&mut board, &mv);
                    }
                }
            } else {
                // if "moves" appears later
                let mut collected: Vec<&str> = Vec::new();
                for tok in parts {
                    collected.push(tok);
                }
                if !collected.is_empty() && collected[0] == "moves" {
                    let mut mv = Move::default();
                    for tok in collected.iter().skip(1) {
                        if parse_move(&board, tok, &mut mv) {
                            make_move(&mut board, &mv);
                        }
                    }
                }
            }
        } else if cmd == "go" {
            let mut search_depth: i32 = unsafe { UCI_OPTIONS.depth };
            let mut move_time: i32 = 0;
            let mut wtime: i32 = 0;
            let mut btime: i32 = 0;
            let mut winc: i32 = 0;
            let mut binc: i32 = 0;
            let mut movestogo: i32 = 40;
            let mut infinite = false;

            while let Some(token) = parts.next() {
                match token {
                    "depth" => {
                        if let Some(ds) = parts.next() {
                            if let Ok(dv) = ds.parse::<i32>() {
                                search_depth = max(1, min(30, dv));
                            }
                        }
                    }
                    "movetime" => {
                        if let Some(ms) = parts.next() {
                            if let Ok(mt) = ms.parse::<i32>() {
                                move_time = mt;
                            }
                        }
                    }
                    "wtime" => {
                        if let Some(ws) = parts.next() {
                            if let Ok(wv) = ws.parse::<i32>() {
                                wtime = wv;
                            }
                        }
                    }
                    "btime" => {
                        if let Some(bs) = parts.next() {
                            if let Ok(bv) = bs.parse::<i32>() {
                                btime = bv;
                            }
                        }
                    }
                    "winc" => {
                        if let Some(ws) = parts.next() {
                            if let Ok(wv) = ws.parse::<i32>() {
                                winc = wv;
                            }
                        }
                    }
                    "binc" => {
                        if let Some(bs) = parts.next() {
                            if let Ok(bv) = bs.parse::<i32>() {
                                binc = bv;
                            }
                        }
                    }
                    "movestogo" => {
                        if let Some(ms) = parts.next() {
                            if let Ok(mv) = ms.parse::<i32>() {
                                movestogo = mv;
                            }
                        }
                    }
                    "infinite" => {
                        infinite = true;
                        search_depth = 20;
                    }
                    _ => {}
                }
            }

            let mut allocated_time = 0;
            if !infinite && move_time == 0 && (wtime > 0 || btime > 0) {
                let time_left = if board.side == WHITE { wtime } else { btime };
                let increment = if board.side == WHITE { winc } else { binc };
                allocated_time = (time_left / movestogo) + (increment as f32 * 0.8) as i32;
                allocated_time = min(allocated_time, time_left / 3);
            } else if move_time > 0 {
                allocated_time = (move_time as f32 * 0.95) as i32;
            }

            let mut best_mv = Move::default();
            unsafe {
                if SEARCH_STATS.is_none() {
                    SEARCH_STATS = Some(SearchStats::new());
                }
                SEARCH_STATS.as_mut().unwrap().init();
            }
            iterative_deepening(&board, search_depth, &mut best_mv, allocated_time);

            if best_mv.from != best_mv.to || best_mv.from != 0 {
                let mut move_str = String::new();
                move_str.push(((best_mv.from % 8) as u8 + b'a') as char);
                move_str.push(((best_mv.from / 8) as u8 + b'1') as char);
                move_str.push(((best_mv.to % 8) as u8 + b'a') as char);
                move_str.push(((best_mv.to / 8) as u8 + b'1') as char);
                if best_mv.promo != 0 {
                    match best_mv.promo {
                        QUEEN => move_str.push('q'),
                        ROOK => move_str.push('r'),
                        BISHOP => move_str.push('b'),
                        KNIGHT => move_str.push('n'),
                        _ => {}
                    }
                }
                println!("bestmove {}", move_str);
            } else {
                let moves = generate_moves(&board, false);
                if !moves.is_empty() {
                    let fallback = moves[0];
                    let mut move_str = String::new();
                    move_str.push(((fallback.from % 8) as u8 + b'a') as char);
                    move_str.push(((fallback.from / 8) as u8 + b'1') as char);
                    move_str.push(((fallback.to % 8) as u8 + b'a') as char);
                    move_str.push(((fallback.to / 8) as u8 + b'1') as char);
                    if fallback.promo != 0 {
                        match fallback.promo {
                            QUEEN => move_str.push('q'),
                            ROOK => move_str.push('r'),
                            BISHOP => move_str.push('b'),
                            KNIGHT => move_str.push('n'),
                            _ => {}
                        }
                    }
                    println!("bestmove {}", move_str);
                } else {
                    println!("bestmove 0000");
                }
            }
        } else if cmd == "quit" {
            break;
        }
    }
}