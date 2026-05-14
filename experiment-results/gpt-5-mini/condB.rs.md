use std::cmp::{max, min};
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

const INF: i32 = 999_999;
const MATE: i32 = 100_000;
const MAX_QUIESCENCE_DEPTH: i32 = 6;
const MAX_PLY: usize = 128;

const TT_SIZE: usize = 1 << 20; // 1M entries

static mut KING_MOVES: [U64; 64] = [0; 64];
static mut KNIGHT_MOVES: [U64; 64] = [0; 64];

static mut ZOBRIST_PIECES: [[[U64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
static mut ZOBRIST_CASTLE: [U64; 16] = [0; 16];
static mut ZOBRIST_EP: [U64; 64] = [0; 64];
static mut ZOBRIST_SIDE: U64 = 0;

#[derive(Copy, Clone)]
struct HistoryTable {
    // [side][from][to]
    scores: [[[i32; 64]; 64]; 2],
}
impl HistoryTable {
    const fn new() -> Self {
        Self {
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
static mut HISTORY_TABLE: HistoryTable = HistoryTable::new();

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Move {
    from: usize,
    to: usize,
    score: i32,
    piece: usize,
    captured: i32,
    promo: usize,
}
impl Move {
    const fn new(f: usize, t: usize, p: usize, c: i32, pr: usize) -> Self {
        Self {
            from: f,
            to: t,
            piece: p,
            captured: c,
            promo: pr,
            score: 0,
        }
    }
}
impl Default for Move {
    fn default() -> Self {
        Self::new(0, 0, 0, -1, 0)
    }
}

#[derive(Copy, Clone)]
struct KillerMoves {
    killers: [[Option<Move>; 2]; MAX_PLY],
}
impl KillerMoves {
    const fn new() -> Self {
        const NONE_PAIR: [Option<Move>; 2] = [None, None];
        Self {
            killers: [NONE_PAIR; MAX_PLY],
        }
    }
    fn init(&mut self) {
        for ply in 0..MAX_PLY {
            self.killers[ply][0] = None;
            self.killers[ply][1] = None;
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
static mut KILLER_MOVES: KillerMoves = KillerMoves::new();

#[derive(Copy, Clone)]
struct TTEntry {
    hash: U64,
    depth: i32,
    score: i32,
    flag: i32,
    best_move: i32,
}
impl TTEntry {
    const fn new() -> Self {
        Self {
            hash: 0,
            depth: 0,
            score: 0,
            flag: 0,
            best_move: 0,
        }
    }
}
const TT_ENTRY_INIT: TTEntry = TTEntry::new();
static mut TRANSPOSITION_TABLE: [TTEntry; TT_SIZE] = [TT_ENTRY_INIT; TT_SIZE];

const TT_EXACT: i32 = 0;
const TT_ALPHA: i32 = 1;
const TT_BETA: i32 = 2;

#[derive(Copy, Clone)]
struct UCIOptions {
    depth: i32,
    use_quiescence: bool,
    quiescence_depth: i32,
}
impl Default for UCIOptions {
    fn default() -> Self {
        Self {
            depth: 8,
            use_quiescence: true,
            quiescence_depth: 4,
        }
    }
}
static mut UCI_OPTIONS: UCIOptions = UCIOptions {
    depth: 8,
    use_quiescence: true,
    quiescence_depth: 4,
};

#[derive(Copy, Clone)]
struct SearchStats {
    nodes: i64,
    qnodes: i64,
    current_depth: i32,
    start_time: Option<Instant>,
}
impl SearchStats {
    fn init(&mut self) {
        self.nodes = 0;
        self.qnodes = 0;
        self.current_depth = 0;
        self.start_time = Some(Instant::now());
    }
    fn nps(&self) -> i64 {
        let elapsed = match self.start_time {
            Some(t) => Instant::now().duration_since(t),
            None => return 0,
        };
        let ms = elapsed.as_millis() as i64;
        if ms == 0 {
            return 0;
        }
        (self.nodes + self.qnodes) * 1000 / ms
    }
}
static mut SEARCH_STATS: SearchStats = SearchStats {
    nodes: 0,
    qnodes: 0,
    current_depth: 0,
    start_time: None,
};

#[derive(Clone)]
struct Board {
    pieces: [[U64; 6]; 2],
    occupied: [U64; 2],
    all: U64,
    side: usize,
    ep: i32,
    castle: i32,
    hash: U64,
}
impl Board {
    fn new() -> Self {
        Self {
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
        unsafe {
            self.hash = zobrist_hash(self);
        }
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
                eval += if c == WHITE { count } else { -count } * values[p];
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
            let sq = wpawns.trailing_zeros() as i32;
            let rank = sq / 8;
            if rank >= 4 {
                eval += (rank - 3) * 15;
            }
            wpawns &= wpawns - 1;
        }

        let mut bpawns = self.pieces[BLACK][PAWN];
        while bpawns != 0 {
            let sq = bpawns.trailing_zeros() as i32;
            let rank = sq / 8;
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

fn init_zobrist() {
    struct SimpleRng {
        state: u64,
    }
    impl SimpleRng {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }
        fn next_u32(&mut self) -> u32 {
            // Xorshift64*
            let mut x = self.state;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.state = x;
            ((x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) & 0xFFFF_FFFF) as u32
        }
    }

    let mut rng = SimpleRng::new(12345);
    unsafe {
        for c in 0..2 {
            for p in 0..6 {
                for sq in 0..64 {
                    let a = (rng.next_u32() as U64) << 48;
                    let b = (rng.next_u32() as U64) << 32;
                    let cval = (rng.next_u32() as U64) << 16;
                    let d = rng.next_u32() as U64;
                    ZOBRIST_PIECES[c][p][sq] = a | b | cval | d;
                }
            }
        }

        for i in 0..16 {
            let a = (rng.next_u32() as U64) << 48;
            let b = (rng.next_u32() as U64) << 32;
            let cval = (rng.next_u32() as U64) << 16;
            let d = rng.next_u32() as U64;
            ZOBRIST_CASTLE[i] = a | b | cval | d;
        }

        for i in 0..64 {
            let a = (rng.next_u32() as U64) << 48;
            let b = (rng.next_u32() as U64) << 32;
            let cval = (rng.next_u32() as U64) << 16;
            let d = rng.next_u32() as U64;
            ZOBRIST_EP[i] = a | b | cval | d;
        }

        let a = (rng.next_u32() as U64) << 48;
        let b = (rng.next_u32() as U64) << 32;
        let cval = (rng.next_u32() as U64) << 16;
        let d = rng.next_u32() as U64;
        ZOBRIST_SIDE = a | b | cval | d;
    }
}

fn zobrist_hash(b: &Board) -> U64 {
    let mut hash: U64 = 0;
    unsafe {
        for c in 0..2 {
            for p in 0..6 {
                let mut bb = b.pieces[c][p];
                while bb != 0 {
                    let sq = bb.trailing_zeros() as usize;
                    hash ^= ZOBRIST_PIECES[c][p][sq];
                    bb &= bb - 1;
                }
            }
        }

        hash ^= ZOBRIST_CASTLE[b.castle as usize];
        if b.ep != -1 {
            hash ^= ZOBRIST_EP[b.ep as usize];
        }
        if b.side == BLACK {
            hash ^= ZOBRIST_SIDE;
        }
    }
    hash
}

fn get_rook_attacks(sq: usize, blockers: U64) -> U64 {
    let mut attacks: U64 = 0;
    let tr = (sq / 8) as i32;
    let tf = (sq % 8) as i32;

    // up
    for r in (tr + 1)..=7 {
        let pos = (r * 8 + tf) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
    }
    // down
    for r in (0..tr).rev() {
        let pos = (r * 8 + tf) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
    }
    // right
    for f in (tf + 1)..=7 {
        let pos = (tr * 8 + f) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
    }
    // left
    for f in (0..tf).rev() {
        let pos = (tr * 8 + f) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
    }
    attacks
}

fn get_bishop_attacks(sq: usize, blockers: U64) -> U64 {
    let mut attacks: U64 = 0;
    let tr = (sq / 8) as i32;
    let tf = (sq % 8) as i32;

    let mut r = tr + 1;
    let mut f = tf + 1;
    while r <= 7 && f <= 7 {
        let pos = (r * 8 + f) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    let mut r = tr + 1;
    let mut f = tf - 1;
    while r <= 7 && f >= 0 {
        let pos = (r * 8 + f) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    let mut r = tr - 1;
    let mut f = tf + 1;
    while r >= 0 && f <= 7 {
        let pos = (r * 8 + f) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    let mut r = tr - 1;
    let mut f = tf - 1;
    while r >= 0 && f >= 0 {
        let pos = (r * 8 + f) as usize;
        attacks |= 1u64 << pos;
        if (1u64 << pos) & blockers != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    attacks
}

fn is_attacked(sq: usize, attacker: usize, b: &Board) -> bool {
    unsafe {
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

        if (KNIGHT_MOVES[sq] & b.pieces[attacker][KNIGHT]) != 0 {
            return true;
        }
        if (KING_MOVES[sq] & b.pieces[attacker][KING]) != 0 {
            return true;
        }
    }

    if (get_rook_attacks(sq, b.all) & (b.pieces[attacker][ROOK] | b.pieces[attacker][QUEEN])) != 0 {
        return true;
    }
    if (get_bishop_attacks(sq, b.all) & (b.pieces[attacker][BISHOP] | b.pieces[attacker][QUEEN])) != 0 {
        return true;
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
        b.hash ^= ZOBRIST_PIECES[b.side][m.piece][m.from];
        b.hash ^= ZOBRIST_PIECES[b.side][m.piece][m.to];

        if b.ep != -1 {
            b.hash ^= ZOBRIST_EP[b.ep as usize];
        }
        b.hash ^= ZOBRIST_CASTLE[b.castle as usize];
    }

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

    unsafe {
        b.hash ^= ZOBRIST_CASTLE[b.castle as usize];
    }
    b.ep = -1;

    b.pieces[b.side][m.piece] ^= from_bb | to_bb;

    // handle captures
    for p in PAWN..=KING {
        if (b.pieces[opponent][p] & to_bb) != 0 {
            b.pieces[opponent][p] ^= to_bb;
            unsafe {
                b.hash ^= ZOBRIST_PIECES[opponent][p][m.to];
            }
            break;
        }
    }

    if m.piece == PAWN {
        if m.to as i32 == prev_ep {
            let captured_pawn_sq = if b.side == WHITE {
                (m.to as i32) - 8
            } else {
                (m.to as i32) + 8
            } as usize;
            b.pieces[opponent][PAWN] ^= 1u64 << captured_pawn_sq;
            unsafe {
                b.hash ^= ZOBRIST_PIECES[opponent][PAWN][captured_pawn_sq];
            }
        }
        if (m.from as i32 - m.to as i32).abs() == 16 {
            b.ep = if b.side == WHITE {
                (m.from as i32) + 8
            } else {
                (m.from as i32) - 8
            };
            unsafe {
                b.hash ^= ZOBRIST_EP[b.ep as usize];
            }
        }
        if m.promo != 0 {
            b.pieces[b.side][PAWN] ^= to_bb;
            b.pieces[b.side][m.promo] ^= to_bb;
            unsafe {
                b.hash ^= ZOBRIST_PIECES[b.side][PAWN][m.to];
                b.hash ^= ZOBRIST_PIECES[b.side][m.promo][m.to];
            }
        }
    } else if m.piece == KING {
        if (m.from as i32 - m.to as i32).abs() == 2 {
            if m.to == 6 {
                b.pieces[WHITE][ROOK] ^= (1u64 << 7) | (1u64 << 5);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[WHITE][ROOK][7];
                    b.hash ^= ZOBRIST_PIECES[WHITE][ROOK][5];
                }
            } else if m.to == 2 {
                b.pieces[WHITE][ROOK] ^= (1u64 << 0) | (1u64 << 3);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[WHITE][ROOK][0];
                    b.hash ^= ZOBRIST_PIECES[WHITE][ROOK][3];
                }
            } else if m.to == 62 {
                b.pieces[BLACK][ROOK] ^= (1u64 << 63) | (1u64 << 61);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[BLACK][ROOK][63];
                    b.hash ^= ZOBRIST_PIECES[BLACK][ROOK][61];
                }
            } else if m.to == 58 {
                b.pieces[BLACK][ROOK] ^= (1u64 << 56) | (1u64 << 59);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[BLACK][ROOK][56];
                    b.hash ^= ZOBRIST_PIECES[BLACK][ROOK][59];
                }
            }
        }
    }

    b.update();
    b.side = opponent;
    unsafe {
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
    let mut moves: Vec<Move> = Vec::with_capacity(if captures_only { 32 } else { 128 });

    for p in PAWN..=KING {
        let mut bitboard = b.pieces[b.side][p];
        while bitboard != 0 {
            let from = bitboard.trailing_zeros() as usize;
            let mut attacks: U64 = 0;

            if p == PAWN {
                let dir: i32 = if b.side == WHITE { 8 } else { -8 };
                let promo_rank: i32 = if b.side == WHITE { 7 } else { 0 };

                if !captures_only {
                    let to_sq = from as i32 + dir;
                    if to_sq >= 0 && to_sq < 64 && (b.all & (1u64 << to_sq)) == 0 {
                        if (to_sq / 8) == promo_rank {
                            moves.push(Move::new(from, to_sq as usize, p, -1, QUEEN));
                        } else {
                            moves.push(Move::new(from, to_sq as usize, p, -1, 0));
                            let start_rank = if b.side == WHITE { 1 } else { 6 };
                            if (from / 8) == start_rank {
                                let to_sq2 = from as i32 + 2 * dir;
                                if (b.all & (1u64 << to_sq2)) == 0 {
                                    moves.push(Move::new(from, to_sq2 as usize, p, -1, 0));
                                }
                            }
                        }
                    }
                }

                let cap_dirs = [dir - 1, dir + 1];
                for &d in &cap_dirs {
                    let to = from as i32 + d;
                    if to < 0 || to > 63 || ((from % 8) as i32 - (to as i32 % 8) as i32).abs() > 1 {
                        continue;
                    }
                    let to_us = to as usize;
                    if (b.occupied[1 - b.side] & (1u64 << to_us)) != 0 {
                        if (to / 8) == promo_rank {
                            moves.push(Move::new(from, to_us, p, 0, QUEEN));
                        } else {
                            moves.push(Move::new(from, to_us, p, -1, 0));
                        }
                    } else if !captures_only && to as i32 == b.ep {
                        moves.push(Move::new(from, to_us, p, -1, 0));
                    }
                }
            } else if p == KING && !captures_only {
                unsafe {
                    attacks = KING_MOVES[from] & !b.occupied[b.side];
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
                        attacks = KNIGHT_MOVES[from];
                    }
                } else if p == BISHOP {
                    attacks = get_bishop_attacks(from, b.all);
                } else if p == ROOK {
                    attacks = get_rook_attacks(from, b.all);
                } else if p == QUEEN {
                    attacks = get_rook_attacks(from, b.all) | get_bishop_attacks(from, b.all);
                } else if p == KING {
                    unsafe {
                        attacks = KING_MOVES[from];
                    }
                }

                if captures_only {
                    attacks &= b.occupied[1 - b.side];
                } else {
                    attacks &= !b.occupied[b.side];
                }
            }

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                moves.push(Move::new(from, to, p, -1, 0));
                attacks &= attacks - 1;
            }

            bitboard &= bitboard - 1;
        }
    }

    let mut legal_moves: Vec<Move> = Vec::with_capacity(moves.len());
    for m in moves {
        if is_legal_move(b, &m) {
            legal_moves.push(m);
        }
    }
    legal_moves
}

fn score_moves(moves: &mut [Move], b: &Board, tt_move: Option<&Move>, ply: usize) {
    for m in moves.iter_mut() {
        if let Some(ttm) = tt_move {
            if *m == *ttm {
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
        } else {
            unsafe {
                if KILLER_MOVES.is_killer(m, ply) {
                    m.score = 90_000;
                } else {
                    m.score = HISTORY_TABLE.get(b.side, m.from, m.to);
                }
            }
        }

        if m.promo == QUEEN {
            m.score += 80_000;
        }
    }

    moves.sort_by(|a, b| b.score.cmp(&a.score));
}

fn quiescence(b: &Board, mut alpha: i32, beta: i32, depth: i32) -> i32 {
    unsafe {
        SEARCH_STATS.qnodes += 1;
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

    for m in captures {
        let mut gain = 200;
        if m.piece != PAWN {
            gain = 900;
        }
        if stand_pat + gain < alpha && depth < -1 {
            continue;
        }

        let mut copy = b.clone();
        make_move(&mut copy, &m);

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
    ply: i32,
    null_move: bool,
) -> i32 {
    unsafe {
        SEARCH_STATS.nodes += 1;
    }

    let mut depth = depth;
    let in_check = is_in_check(b);
    if in_check {
        depth += 1;
    }

    let tt_index = (b.hash as usize) % TT_SIZE;
    let mut tt_move_opt: Option<Move> = None;
    unsafe {
        let tt_entry = TRANSPOSITION_TABLE[tt_index];
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
            let mut mv = Move::default();
            mv.from = (tt_entry.best_move & 63) as usize;
            mv.to = ((tt_entry.best_move >> 6) & 63) as usize;
            mv.piece = ((tt_entry.best_move >> 12) & 7) as usize;
            tt_move_opt = Some(mv);
        }
    }

    if depth <= 0 {
        return quiescence(b, alpha, beta, 0);
    }

    if null_move && !in_check && depth >= 3 && ply > 0 {
        let mut copy = b.clone();
        copy.side = 1 - copy.side;
        unsafe {
            copy.hash ^= ZOBRIST_SIDE;
        }
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
            return -MATE + ply;
        }
        return 0;
    }

    score_moves(
        &mut moves,
        b,
        tt_move_opt.as_ref(),
        ply as usize,
    );

    if ply == 0 && !moves.is_empty() {
        *best_move = moves[0];
    }

    let mut move_count = 0;
    let mut best_score = -INF;
    let mut local_best = Move::default();
    let orig_alpha = alpha;

    for m in moves.into_iter() {
        move_count += 1;

        let mut reduction = 0;
        if move_count > 4
            && depth >= 3
            && !in_check
            && (b.occupied[1 - b.side] & (1u64 << m.to)) == 0
            && m.promo == 0
        {
            if move_count > 12 {
                reduction = 3;
            } else if move_count > 6 {
                reduction = 2;
            } else {
                reduction = 1;
            }
            unsafe {
                if KILLER_MOVES.is_killer(&m, ply as usize)
                    || HISTORY_TABLE.get(b.side, m.from, m.to) > 5000
                {
                    reduction = max(0, reduction - 1);
                }
            }
        }

        let mut copy = b.clone();
        make_move(&mut copy, &m);

        let score: i32;
        if move_count == 1 {
            let mut dummy = Move::default();
            score = -search(&copy, depth - 1 - reduction, -beta, -alpha, &mut dummy, ply + 1, true);
        } else {
            let mut dummy = Move::default();
            let mut sc = -search(&copy, depth - 1 - reduction, -alpha - 1, -alpha, &mut dummy, ply + 1, true);
            if sc > alpha && sc < beta {
                let mut dummy2 = Move::default();
                sc = -search(&copy, depth - 1, -beta, -alpha, &mut dummy2, ply + 1, true);
            }
            // Reassign
            let mut sc_final = sc;
            // Re-search without reduction if reduced search failed high
            if reduction > 0 && sc_final > alpha {
                let mut dummy3 = Move::default();
                sc_final = -search(&copy, depth - 1, -beta, -alpha, &mut dummy3, ply + 1, true);
            }
            score = sc_final;
        }

        if score > best_score {
            best_score = score;
            local_best = m;
            if ply == 0 {
                *best_move = m;
            }
        }

        if score > alpha {
            alpha = score;
            if (b.occupied[1 - b.side] & (1u64 << m.to)) == 0 {
                unsafe {
                    HISTORY_TABLE.update(b.side, m.from, m.to, depth);
                }
            }
        }

        if alpha >= beta {
            if (b.occupied[1 - b.side] & (1u64 << m.to)) == 0 {
                unsafe {
                    KILLER_MOVES.update(m, ply as usize);
                }
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

    unsafe {
        let tt_entry = &mut TRANSPOSITION_TABLE[tt_index];
        tt_entry.hash = b.hash;
        tt_entry.depth = depth;
        tt_entry.score = best_score;
        tt_entry.best_move =
            (local_best.from as i32) | ((local_best.to as i32) << 6) | ((local_best.piece as i32) << 12);

        if best_score <= orig_alpha {
            tt_entry.flag = TT_ALPHA;
        } else if best_score >= beta {
            tt_entry.flag = TT_BETA;
        } else {
            tt_entry.flag = TT_EXACT;
        }
    }

    best_score
}

fn iterative_deepening(b: &Board, max_depth: i32, best_move: &mut Move, time_limit: i32) -> i32 {
    let mut score = 0;
    let mut alpha = -INF;
    let mut beta = INF;
    let mut window = 50;

    unsafe {
        SEARCH_STATS.init();
    }

    for depth in 1..=max_depth {
        unsafe {
            SEARCH_STATS.current_depth = depth;
        }

        if depth >= 4 {
            alpha = score - window;
            beta = score + window;
        }

        let temp_score = {
            let mut bm = Move::default();
            search(b, depth, alpha, beta, &mut bm, 0, true)
        };

        let mut final_score = temp_score;
        if final_score <= alpha || final_score >= beta {
            let mut bm = Move::default();
            final_score = search(b, depth, -INF, INF, &mut bm, 0, true);
            window = 50;
        } else {
            window = 25;
        }

        score = final_score;

        if time_limit > 0 {
            unsafe {
                if let Some(start) = SEARCH_STATS.start_time {
                    let elapsed = Instant::now().duration_since(start);
                    let ms = elapsed.as_millis() as i32;
                    if ms > (time_limit as i32) * 40 / 100 && depth > 4 {
                        break;
                    }
                }
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
            print!(" nodes {}", SEARCH_STATS.nodes);
            print!(" nps {}", SEARCH_STATS.nps());
        }
        print!(" pv ");

        let mut move_str = String::new();
        move_str.push((('a' as u8) + (best_move.from % 8) as u8) as char);
        move_str.push((('1' as u8) + (best_move.from / 8) as u8) as char);
        move_str.push((('a' as u8) + (best_move.to % 8) as u8) as char);
        move_str.push((('1' as u8) + (best_move.to / 8) as u8) as char);
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

fn init_tables() {
    for sq in 0..64 {
        let x = (sq % 8) as i32;
        let y = (sq / 8) as i32;

        let mut km: U64 = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    km |= 1u64 << (ny * 8 + nx);
                }
            }
        }
        unsafe {
            KING_MOVES[sq] = km;
        }

        let mut nm: U64 = 0;
        let kdx = [2, 2, -2, -2, 1, 1, -1, -1];
        let kdy = [1, -1, 1, -1, 2, -2, 2, -2];
        for i in 0..8 {
            let nx = x + kdx[i];
            let ny = y + kdy[i];
            if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                nm |= 1u64 << (ny * 8 + nx);
            }
        }
        unsafe {
            KNIGHT_MOVES[sq] = nm;
        }
    }

    init_zobrist();
    unsafe {
        HISTORY_TABLE.init();
        KILLER_MOVES.init();
        // TRANSPOSITION_TABLE already zero-initialized statically
    }
}

fn parse_move(b: &Board, move_str: &str, parsed_move: &mut Move) -> bool {
    let moves = generate_moves(b, false);
    if move_str.len() < 4 {
        return false;
    }
    let from = (move_str.as_bytes()[0] - b'a') as usize + ((move_str.as_bytes()[1] - b'1') as usize) * 8;
    let to = (move_str.as_bytes()[2] - b'a') as usize + ((move_str.as_bytes()[3] - b'1') as usize) * 8;
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

    for m in moves {
        if m.from == from && m.to == to {
            if m.promo != 0 {
                if m.promo == promo_piece {
                    *parsed_move = m;
                    return true;
                }
            } else {
                *parsed_move = m;
                return true;
            }
        }
    }
    false
}

fn format_move_str(mv: &Move) -> String {
    let mut s = String::new();
    s.push((('a' as u8) + (mv.from % 8) as u8) as char);
    s.push((('1' as u8) + (mv.from / 8) as u8) as char);
    s.push((('a' as u8) + (mv.to % 8) as u8) as char);
    s.push((('1' as u8) + (mv.to / 8) as u8) as char);
    if mv.promo != 0 {
        match mv.promo {
            QUEEN => s.push('q'),
            ROOK => s.push('r'),
            BISHOP => s.push('b'),
            KNIGHT => s.push('n'),
            _ => {}
        }
    }
    s
}

fn main() {
    init_tables();
    let mut board = Board::new();
    board.init();

    let stdin = io::stdin();
    for line_res in stdin.lock().lines() {
        if line_res.is_err() {
            break;
        }
        let line = line_res.unwrap();
        if line.is_empty() {
            continue;
        }
        let mut parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let cmd = parts[0];
        if cmd == "uci" {
            println!("id name NanoChessTurbo");
            println!("id author CrvProject");
            println!("option name Depth type spin default 10 min 1 max 30");
            println!("option name Hash type spin default 64 min 1 max 1024");
            println!("uciok");
        } else if cmd == "setoption" {
            // naive parsing: find "name" then option name then optional "value"
            let mut i = 1;
            let mut name = String::new();
            while i < parts.len() {
                if parts[i] == "name" {
                    i += 1;
                    while i < parts.len() && parts[i] != "value" {
                        if !name.is_empty() {
                            name.push(' ');
                        }
                        name.push_str(parts[i]);
                        i += 1;
                    }
                    break;
                }
                i += 1;
            }
            if name == "Depth" {
                // find value token if present
                if let Some(pos) = parts.iter().position(|&s| s == "value") {
                    if pos + 1 < parts.len() {
                        if let Ok(v) = parts[pos + 1].parse::<i32>() {
                            unsafe {
                                UCI_OPTIONS.depth = max(1, min(30, v));
                            }
                        }
                    }
                }
            } else if name == "Hash" {
                // ignore for now
            }
        } else if cmd == "isready" {
            println!("readyok");
        } else if cmd == "ucinewgame" {
            board.init();
            unsafe {
                HISTORY_TABLE.init();
                KILLER_MOVES.init();
                for i in 0..TT_SIZE {
                    TRANSPOSITION_TABLE[i] = TT_ENTRY_INIT;
                }
            }
        } else if cmd == "position" {
            // position [startpos|fen ...] moves ...
            let mut idx = 1;
            if idx < parts.len() && parts[idx] == "startpos" {
                board.init();
                idx += 1;
            } else if idx < parts.len() && parts[idx] == "fen" {
                // skip fen fields until "moves" or end
                idx += 1;
                while idx < parts.len() && parts[idx] != "moves" {
                    idx += 1;
                }
            }

            if idx < parts.len() && parts[idx] == "moves" {
                idx += 1;
                while idx < parts.len() {
                    let token = parts[idx];
                    let mut m = Move::default();
                    if parse_move(&board, token, &mut m) {
                        make_move(&mut board, &m);
                    }
                    idx += 1;
                }
            }
        } else if cmd == "go" {
            let mut search_depth: i32;
            unsafe {
                search_depth = UCI_OPTIONS.depth;
            }
            let mut move_time = 0i32;
            let mut wtime = 0i32;
            let mut btime = 0i32;
            let mut winc = 0i32;
            let mut binc = 0i32;
            let mut movestogo = 40i32;
            let mut infinite = false;

            let mut i = 1;
            while i < parts.len() {
                match parts[i] {
                    "depth" => {
                        if i + 1 < parts.len() {
                            if let Ok(v) = parts[i + 1].parse::<i32>() {
                                search_depth = max(1, min(30, v));
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "movetime" => {
                        if i + 1 < parts.len() {
                            move_time = parts[i + 1].parse::<i32>().unwrap_or(0);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "wtime" => {
                        if i + 1 < parts.len() {
                            wtime = parts[i + 1].parse::<i32>().unwrap_or(0);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "btime" => {
                        if i + 1 < parts.len() {
                            btime = parts[i + 1].parse::<i32>().unwrap_or(0);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "winc" => {
                        if i + 1 < parts.len() {
                            winc = parts[i + 1].parse::<i32>().unwrap_or(0);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "binc" => {
                        if i + 1 < parts.len() {
                            binc = parts[i + 1].parse::<i32>().unwrap_or(0);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "movestogo" => {
                        if i + 1 < parts.len() {
                            movestogo = parts[i + 1].parse::<i32>().unwrap_or(40);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "infinite" => {
                        infinite = true;
                        search_depth = 20;
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            let mut allocated_time = 0i32;
            if !infinite && move_time == 0 && (wtime > 0 || btime > 0) {
                let time_left = if board.side == WHITE { wtime } else { btime };
                let increment = if board.side == WHITE { winc } else { binc };

                allocated_time = (time_left / movestogo) + (increment as f32 * 0.8) as i32;
                allocated_time = min(allocated_time, time_left / 3);
            } else if move_time > 0 {
                allocated_time = (move_time as f32 * 0.95) as i32;
            }

            let mut best_move = Move::default();
            iterative_deepening(&board, search_depth, &mut best_move, allocated_time);

            if best_move.from != best_move.to || best_move.from != 0 {
                let move_str = format_move_str(&best_move);
                println!("bestmove {}", move_str);
            } else {
                let moves = generate_moves(&board, false);
                if !moves.is_empty() {
                    let fallback = moves[0];
                    let move_str = format_move_str(&fallback);
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