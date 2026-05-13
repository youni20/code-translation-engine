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

const WHITE: i32 = 0;
const BLACK: i32 = 1;

const INF: i32 = 999_999;
const MATE: i32 = 100_000;
const MAX_QUIESCENCE_DEPTH: i32 = 6;
const MAX_PLY: usize = 128;

const TT_SIZE: usize = 1 << 20; // 1,048,576

static mut KING_MOVES: [U64; 64] = [0; 64];
static mut KNIGHT_MOVES: [U64; 64] = [0; 64];

#[derive(Copy, Clone)]
struct HistoryTable {
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

#[derive(Copy, Clone, Default, PartialEq, Eq)]
struct Move {
    from: usize,
    to: usize,
    score: i32,
    piece: i32,
    captured: i32,
    promo: i32,
}
impl Move {
    fn new(f: usize, t: usize, p: i32, c: i32, pr: i32) -> Move {
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

#[derive(Copy, Clone)]
struct KillerMoves {
    killers: [[Option<Move>; 2]; MAX_PLY],
}
impl KillerMoves {
    const fn new() -> Self {
        const NONE_MOVE: Option<Move> = None;
        Self {
            killers: [[NONE_MOVE; 2]; MAX_PLY],
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
impl Default for KillerMoves {
    fn default() -> Self {
        KillerMoves::new()
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
static mut TRANSPOSITION_TABLE: [TTEntry; TT_SIZE] = [TTEntry::new(); TT_SIZE];

const TT_EXACT: i32 = 0;
const TT_ALPHA: i32 = 1;
const TT_BETA: i32 = 2;

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

struct SearchStats {
    nodes: i64,
    qnodes: i64,
    current_depth: i32,
    start_time: Option<Instant>,
}
impl SearchStats {
    fn new() -> Self {
        Self {
            nodes: 0,
            qnodes: 0,
            current_depth: 0,
            start_time: Some(Instant::now()),
        }
    }
    fn init(&mut self) {
        self.nodes = 0;
        self.qnodes = 0;
        self.current_depth = 0;
        self.start_time = Some(Instant::now());
    }
    fn nps(&self) -> i64 {
        let start = match &self.start_time {
            Some(t) => t,
            None => return 0,
        };
        let elapsed = Instant::now() - *start;
        let ms = elapsed.as_millis();
        if ms == 0 {
            return 0;
        }
        let num = ((self.nodes + self.qnodes) * 1000) as i128;
        let denom = ms as i128;
        (num / denom) as i64
    }
}
static mut SEARCH_STATS: SearchStats = SearchStats {
    nodes: 0,
    qnodes: 0,
    current_depth: 0,
    start_time: None,
};

static mut ZOBRIST_PIECES: [[[U64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
static mut ZOBRIST_CASTLE: [U64; 16] = [0; 16];
static mut ZOBRIST_EP: [U64; 64] = [0; 64];
static mut ZOBRIST_SIDE: U64 = 0;

fn lcg_rand(state: &mut u64) -> u64 {
    // Simple LCG for reproducible numbers
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state
}

fn init_zobrist() {
    unsafe {
        let mut seed: u64 = 12345;
        for c in 0..2 {
            for p in 0..6 {
                for sq in 0..64 {
                    let r = lcg_rand(&mut seed);
                    ZOBRIST_PIECES[c][p][sq] = r;
                }
            }
        }
        for i in 0..16 {
            ZOBRIST_CASTLE[i] = lcg_rand(&mut seed);
        }
        for i in 0..64 {
            ZOBRIST_EP[i] = lcg_rand(&mut seed);
        }
        ZOBRIST_SIDE = lcg_rand(&mut seed);
    }
}

#[derive(Clone)]
struct Board {
    pieces: [[U64; 6]; 2],
    occupied: [U64; 2],
    all: U64,
    side: i32,
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
        self.pieces[WHITE as usize][PAWN] = 0xFF00u64;
        self.pieces[WHITE as usize][KNIGHT] = 0x42u64;
        self.pieces[WHITE as usize][BISHOP] = 0x24u64;
        self.pieces[WHITE as usize][ROOK] = 0x81u64;
        self.pieces[WHITE as usize][QUEEN] = 0x8u64;
        self.pieces[WHITE as usize][KING] = 0x10u64;

        self.pieces[BLACK as usize][PAWN] = 0xFF000000000000u64;
        self.pieces[BLACK as usize][KNIGHT] = 0x4200000000000000u64;
        self.pieces[BLACK as usize][BISHOP] = 0x2400000000000000u64;
        self.pieces[BLACK as usize][ROOK] = 0x8100000000000000u64;
        self.pieces[BLACK as usize][QUEEN] = 0x800000000000000u64;
        self.pieces[BLACK as usize][KING] = 0x1000000000000000u64;

        self.update();
        self.side = WHITE;
        self.ep = -1;
        self.castle = 15;
        unsafe {
            self.hash = zobrist_hash(self);
        }
    }
    fn update(&mut self) {
        self.occupied[WHITE as usize] = 0;
        self.occupied[BLACK as usize] = 0;
        for p in PAWN..=KING {
            self.occupied[WHITE as usize] |= self.pieces[WHITE as usize][p];
            self.occupied[BLACK as usize] |= self.pieces[BLACK as usize][p];
        }
        self.all = self.occupied[WHITE as usize] | self.occupied[BLACK as usize];
    }
    fn evaluate(&self) -> i32 {
        let mut eval: i32 = 0;
        let values = [100, 320, 330, 500, 900, 0];

        for c in 0..2 {
            for p in 0..6 {
                let count = self.pieces[c][p].count_ones() as i32;
                if c == WHITE as usize {
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
            let king_sq = self.pieces[c][KING].trailing_zeros() as i32;
            if c == WHITE as usize {
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
        eval += ( (self.pieces[WHITE as usize][PAWN] & center).count_ones() as i32
            - (self.pieces[BLACK as usize][PAWN] & center).count_ones() as i32) * 20;

        let mut wpawns = self.pieces[WHITE as usize][PAWN];
        while wpawns != 0 {
            let sq = wpawns.trailing_zeros() as i32;
            let rank = sq / 8;
            if rank >= 4 {
                eval += (rank - 3) * 15;
            }
            wpawns &= wpawns - 1;
        }

        let mut bpawns = self.pieces[BLACK as usize][PAWN];
        while bpawns != 0 {
            let sq = bpawns.trailing_zeros() as i32;
            let rank = sq / 8;
            if rank <= 3 {
                eval -= (4 - rank) * 15;
            }
            bpawns &= bpawns - 1;
        }

        if self.side == WHITE { eval } else { -eval }
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

    for r in (tr + 1)..=7 {
        let idx = (r * 8 + tf) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
    }
    for r in (0..tr).rev() {
        let idx = (r * 8 + tf) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
    }
    for f in (tf + 1)..=7 {
        let idx = (tr * 8 + f) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
    }
    for f in (0..tf).rev() {
        let idx = (tr * 8 + f) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
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
        let idx = (r * 8 + f) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    r = tr + 1;
    f = tf - 1;
    while r <= 7 && f >= 0 {
        let idx = (r * 8 + f) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    r = tr - 1;
    f = tf + 1;
    while r >= 0 && f <= 7 {
        let idx = (r * 8 + f) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    r = tr - 1;
    f = tf - 1;
    while r >= 0 && f >= 0 {
        let idx = (r * 8 + f) as usize;
        attacks |= 1u64 << idx;
        if (1u64 << idx) & blockers != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    attacks
}

fn is_attacked(sq: usize, attacker: i32, b: &Board) -> bool {
    unsafe {
        if attacker == WHITE {
            if sq >= 9 && sq % 8 != 0 && ((1u64 << (sq - 9)) & b.pieces[WHITE as usize][PAWN]) != 0 {
                return true;
            }
            if sq >= 7 && sq % 8 != 7 && ((1u64 << (sq - 7)) & b.pieces[WHITE as usize][PAWN]) != 0 {
                return true;
            }
        } else {
            if sq <= 56 && sq % 8 != 0 && ((1u64 << (sq + 7)) & b.pieces[BLACK as usize][PAWN]) != 0 {
                return true;
            }
            if sq <= 54 && sq % 8 != 7 && ((1u64 << (sq + 9)) & b.pieces[BLACK as usize][PAWN]) != 0 {
                return true;
            }
        }

        if (KNIGHT_MOVES[sq] & b.pieces[attacker as usize][KNIGHT]) != 0 {
            return true;
        }
        if (KING_MOVES[sq] & b.pieces[attacker as usize][KING]) != 0 {
            return true;
        }
    }

    if (get_rook_attacks(sq, b.all) & (b.pieces[attacker as usize][ROOK] | b.pieces[attacker as usize][QUEEN])) != 0 {
        return true;
    }
    if (get_bishop_attacks(sq, b.all) & (b.pieces[attacker as usize][BISHOP] | b.pieces[attacker as usize][QUEEN])) != 0 {
        return true;
    }

    false
}

fn is_in_check(b: &Board) -> bool {
    let side = b.side as usize;
    if b.pieces[side][KING] == 0 {
        return false;
    }
    let king_sq = b.pieces[side][KING].trailing_zeros() as usize;
    is_attacked(king_sq, 1 - b.side, b)
}

fn make_move(b: &mut Board, m: Move) {
    let from_bb: U64 = 1u64 << m.from;
    let to_bb: U64 = 1u64 << m.to;

    let opponent = 1 - b.side;
    let prev_ep = b.ep;

    unsafe {
        b.hash ^= ZOBRIST_PIECES[b.side as usize][m.piece as usize][m.from];
        b.hash ^= ZOBRIST_PIECES[b.side as usize][m.piece as usize][m.to];

        if b.ep != -1 {
            b.hash ^= ZOBRIST_EP[b.ep as usize];
        }
        b.hash ^= ZOBRIST_CASTLE[b.castle as usize];
    }

    if m.piece == KING as i32 {
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

    // Move piece
    let side_idx = b.side as usize;
    b.pieces[side_idx][m.piece as usize] ^= from_bb | to_bb;

    // Handle captures
    let opponent_idx = opponent as usize;
    for p in PAWN..=KING {
        if (b.pieces[opponent_idx][p] & to_bb) != 0 {
            b.pieces[opponent_idx][p] ^= to_bb;
            unsafe {
                b.hash ^= ZOBRIST_PIECES[opponent_idx][p][m.to];
            }
            break;
        }
    }

    // Special moves
    if m.piece == PAWN as i32 {
        if m.to as i32 == prev_ep {
            let captured_pawn_sq = if b.side == WHITE { (m.to as i32) - 8 } else { (m.to as i32) + 8 };
            let sq = captured_pawn_sq as usize;
            b.pieces[opponent_idx][PAWN] ^= 1u64 << sq;
            unsafe {
                b.hash ^= ZOBRIST_PIECES[opponent_idx][PAWN][sq];
            }
        }
        if (m.from as i32 - m.to as i32).abs() == 16 {
            b.ep = if b.side == WHITE { (m.from as i32) + 8 } else { (m.from as i32) - 8 };
            unsafe {
                b.hash ^= ZOBRIST_EP[b.ep as usize];
            }
        }
        if m.promo != 0 {
            b.pieces[side_idx][PAWN] ^= to_bb;
            b.pieces[side_idx][m.promo as usize] ^= to_bb;
            unsafe {
                b.hash ^= ZOBRIST_PIECES[side_idx][PAWN][m.to];
                b.hash ^= ZOBRIST_PIECES[side_idx][m.promo as usize][m.to];
            }
        }
    } else if m.piece == KING as i32 {
        if (m.from as i32 - m.to as i32).abs() == 2 {
            if m.to == 6 {
                b.pieces[WHITE as usize][ROOK] ^= (1u64 << 7) | (1u64 << 5);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[WHITE as usize][ROOK][7];
                    b.hash ^= ZOBRIST_PIECES[WHITE as usize][ROOK][5];
                }
            } else if m.to == 2 {
                b.pieces[WHITE as usize][ROOK] ^= (1u64 << 0) | (1u64 << 3);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[WHITE as usize][ROOK][0];
                    b.hash ^= ZOBRIST_PIECES[WHITE as usize][ROOK][3];
                }
            } else if m.to == 62 {
                b.pieces[BLACK as usize][ROOK] ^= (1u64 << 63) | (1u64 << 61);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[BLACK as usize][ROOK][63];
                    b.hash ^= ZOBRIST_PIECES[BLACK as usize][ROOK][61];
                }
            } else if m.to == 58 {
                b.pieces[BLACK as usize][ROOK] ^= (1u64 << 56) | (1u64 << 59);
                unsafe {
                    b.hash ^= ZOBRIST_PIECES[BLACK as usize][ROOK][56];
                    b.hash ^= ZOBRIST_PIECES[BLACK as usize][ROOK][59];
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

fn is_legal_move(b: &Board, m: Move) -> bool {
    let mut copy = b.clone();
    make_move(&mut copy, m);
    if copy.pieces[b.side as usize][KING] == 0 {
        return false;
    }
    let king_sq = copy.pieces[b.side as usize][KING].trailing_zeros() as usize;
    !is_attacked(king_sq, copy.side, &copy)
}

fn generate_moves(b: &Board, captures_only: bool) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(if captures_only { 32 } else { 128 });

    for p in PAWN..=KING {
        let mut bitboard = b.pieces[b.side as usize][p];
        while bitboard != 0 {
            let from = bitboard.trailing_zeros() as usize;
            let mut attacks: U64 = 0;

            if p == PAWN {
                let dir = if b.side == WHITE { 8 } else { -8 };
                let promo_rank = if b.side == WHITE { 7 } else { 0 };

                if !captures_only {
                    let to_sq = (from as i32 + dir) as i32;
                    if to_sq >= 0 && to_sq < 64 && (b.all & (1u64 << to_sq)) == 0 {
                        if (to_sq / 8) == promo_rank {
                            moves.push(Move::new(from, to_sq as usize, p as i32, -1, QUEEN as i32));
                        } else {
                            moves.push(Move::new(from, to_sq as usize, p as i32, -1, 0));
                            let start_rank = if b.side == WHITE { 1 } else { 6 };
                            if (from / 8) == start_rank {
                                let to_sq2 = (from as i32 + 2 * dir) as i32;
                                if (b.all & (1u64 << to_sq2)) == 0 {
                                    moves.push(Move::new(from, to_sq2 as usize, p as i32, -1, 0));
                                }
                            }
                        }
                    }
                }

                let cap_dirs = [dir - 1, dir + 1];
                for &d in &cap_dirs {
                    let to_i = from as i32 + d;
                    if to_i < 0 || to_i > 63 || ( (from % 8) as i32 - (to_i as i32 % 8) ).abs() > 1 {
                        continue;
                    }
                    let to = to_i as usize;
                    if (b.occupied[1 - b.side as usize] & (1u64 << to)) != 0 {
                        if (to / 8) as i32 == promo_rank {
                            moves.push(Move::new(from, to, p as i32, 0, QUEEN as i32));
                        } else {
                            moves.push(Move::new(from, to, p as i32, -1, 0));
                        }
                    } else if !captures_only && (to as i32) == b.ep {
                        moves.push(Move::new(from, to, p as i32, -1, 0));
                    }
                }
            } else if p == KING && !captures_only {
                unsafe {
                    attacks = KING_MOVES[from] & !b.occupied[b.side as usize];
                }
                if !is_in_check(b) {
                    if b.side == WHITE {
                        if (b.castle & 1) != 0 && (b.all & 0x60u64) == 0 {
                            if !is_attacked(5, BLACK, b) && !is_attacked(6, BLACK, b) {
                                moves.push(Move::new(4, 6, KING as i32, -1, 0));
                            }
                        }
                        if (b.castle & 2) != 0 && (b.all & 0xEu64) == 0 {
                            if !is_attacked(3, BLACK, b) && !is_attacked(2, BLACK, b) {
                                moves.push(Move::new(4, 2, KING as i32, -1, 0));
                            }
                        }
                    } else {
                        if (b.castle & 4) != 0 && (b.all & 0x6000000000000000u64) == 0 {
                            if !is_attacked(61, WHITE, b) && !is_attacked(62, WHITE, b) {
                                moves.push(Move::new(60, 62, KING as i32, -1, 0));
                            }
                        }
                        if (b.castle & 8) != 0 && (b.all & 0xE00000000000000u64) == 0 {
                            if !is_attacked(59, WHITE, b) && !is_attacked(58, WHITE, b) {
                                moves.push(Move::new(60, 58, KING as i32, -1, 0));
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
                    attacks &= b.occupied[1 - b.side as usize];
                } else {
                    attacks &= !b.occupied[b.side as usize];
                }
            }

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                moves.push(Move::new(from, to, p as i32, -1, 0));
                attacks &= attacks - 1;
            }

            bitboard &= bitboard - 1;
        }
    }

    let mut legal_moves: Vec<Move> = Vec::with_capacity(moves.len());
    for m in moves {
        if is_legal_move(b, m) {
            legal_moves.push(m);
        }
    }
    legal_moves
}

fn score_moves(moves: &mut [Move], b: &Board, tt_move: Option<Move>, ply: usize) {
    for m in moves.iter_mut() {
        if let Some(ttm) = tt_move {
            if *m == ttm {
                m.score = 1_000_000;
                continue;
            }
        }

        if (b.occupied[1 - b.side as usize] & (1u64 << m.to)) != 0 {
            let victim_values = [100, 300, 300, 500, 900, 10000];
            for p in (PAWN..=KING).rev() {
                if (b.pieces[1 - b.side as usize][p] & (1u64 << m.to)) != 0 {
                    m.score = 100_000 + victim_values[p] * 10 - victim_values[m.piece as usize];
                    break;
                }
            }
        } else {
            unsafe {
                if KILLER_MOVES.is_killer(m, ply) {
                    m.score = 90_000;
                    continue;
                }
            }
            unsafe {
                m.score = HISTORY_TABLE.get(b.side as usize, m.from, m.to);
            }
        }
        if m.promo == QUEEN as i32 {
            m.score += 80_000;
        }
    }

    moves.sort_by(|a, b| b.score.cmp(&a.score));
}

fn quiescence(b: &Board, alpha: i32, beta: i32, depth: i32) -> i32 {
    unsafe {
        SEARCH_STATS.qnodes += 1;
    }

    let mut alpha = alpha;
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
        if m.piece != PAWN as i32 {
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

fn search(b: &Board, depth: i32, alpha: i32, beta: i32, best_move: &mut Move, ply: i32, null_move: bool) -> i32 {
    unsafe {
        SEARCH_STATS.nodes += 1;
    }

    let mut depth = depth;
    let in_check = is_in_check(b);
    if in_check {
        depth += 1;
    }

    let tt_index = (b.hash as usize) % TT_SIZE;
    let mut tt_entry: TTEntry;
    unsafe {
        tt_entry = TRANSPOSITION_TABLE[tt_index];
    }
    let mut tt_move: Option<Move> = None;

    if tt_entry.hash == b.hash && tt_entry.depth >= depth {
        if tt_entry.flag == TT_EXACT {
            if ply == 0 {
                best_move.from = (tt_entry.best_move & 63) as usize;
                best_move.to = ((tt_entry.best_move >> 6) & 63) as usize;
                best_move.piece = ((tt_entry.best_move >> 12) & 7) as i32;
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
        let mm = Move::new(
            (tt_entry.best_move & 63) as usize,
            ((tt_entry.best_move >> 6) & 63) as usize,
            ((tt_entry.best_move >> 12) & 7) as i32,
            -1,
            0,
        );
        tt_move = Some(mm);
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

    let tt_move_ref = if tt_entry.hash == b.hash { tt_move } else { None };
    score_moves(&mut moves, b, tt_move_ref, ply as usize);

    if ply == 0 && !moves.is_empty() {
        *best_move = moves[0];
    }

    let mut move_count = 0;
    let mut best_score = -INF;
    let mut local_best = Move::default();
    let orig_alpha = alpha;
    let mut alpha = alpha;

    for m in moves {
        move_count += 1;
        let mut reduction = 0;
        if move_count > 4 && depth >= 3 && !in_check &&
            (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 &&
            m.promo == 0 {
            if move_count > 12 {
                reduction = 3;
            } else if move_count > 6 {
                reduction = 2;
            } else {
                reduction = 1;
            }
            unsafe {
                if KILLER_MOVES.is_killer(&m, ply as usize) || HISTORY_TABLE.get(b.side as usize, m.from, m.to) > 5000 {
                    reduction = max(0, reduction - 1);
                }
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
            score = sc;
        }

        if reduction > 0 && score > alpha {
            let mut dummy = Move::default();
            let sc = -search(&copy, depth - 1, -beta, -alpha, &mut dummy, ply + 1, true);
            let score = sc;
            if score > best_score {
                best_score = score;
                local_best = m;
                if ply == 0 {
                    *best_move = m;
                }
            }
            if score > alpha {
                alpha = score;
                unsafe {
                    if (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 {
                        HISTORY_TABLE.update(b.side as usize, m.from, m.to, depth);
                    }
                }
            }
            if alpha >= beta {
                unsafe {
                    if (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 {
                        KILLER_MOVES.update(m, ply as usize);
                    }
                }
                break;
            }
            // futility pruning check continues
            if depth <= 2 && !in_check && move_count > 8 &&
                (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 {
                let futility_margin = depth * 100;
                if b.evaluate() + futility_margin < alpha {
                    break;
                }
            }
            continue;
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
            unsafe {
                if (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 {
                    HISTORY_TABLE.update(b.side as usize, m.from, m.to, depth);
                }
            }
        }

        if alpha >= beta {
            unsafe {
                if (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 {
                    KILLER_MOVES.update(m, ply as usize);
                }
            }
            break;
        }

        if depth <= 2 && !in_check && move_count > 8 &&
            (b.occupied[1 - b.side as usize] & (1u64 << m.to)) == 0 {
            let futility_margin = depth * 100;
            if b.evaluate() + futility_margin < alpha {
                break;
            }
        }
    }

    // Store in transposition table
    unsafe {
        TRANSPOSITION_TABLE[tt_index].hash = b.hash;
        TRANSPOSITION_TABLE[tt_index].depth = depth;
        TRANSPOSITION_TABLE[tt_index].score = best_score;
        TRANSPOSITION_TABLE[tt_index].best_move = (local_best.from as i32) | ((local_best.to as i32) << 6) | ((local_best.piece as i32) << 12);
        if best_score <= orig_alpha {
            TRANSPOSITION_TABLE[tt_index].flag = TT_ALPHA;
        } else if best_score >= beta {
            TRANSPOSITION_TABLE[tt_index].flag = TT_BETA;
        } else {
            TRANSPOSITION_TABLE[tt_index].flag = TT_EXACT;
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
            let mut bm_local = Move::default();
            search(b, depth, alpha, beta, &mut bm_local, 0, true)
        };

        let mut temp_score = temp_score;
        if temp_score <= alpha || temp_score >= beta {
            let mut bm_local = Move::default();
            temp_score = search(b, depth, -INF, INF, &mut bm_local, 0, true);
            window = 50;
        } else {
            window = 25;
        }

        score = temp_score;

        if time_limit > 0 {
            unsafe {
                if let Some(start) = SEARCH_STATS.start_time {
                    let elapsed = Instant::now() - start;
                    let ms = elapsed.as_millis() as i128;
                    if ms > (time_limit as i128) * 4 / 10 && depth > 4 {
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
            if score < 0 { mate_in = -mate_in; }
            print!("mate {}", mate_in);
        } else {
            print!("cp {}", score);
        }

        unsafe {
            print!(" nodes {}", SEARCH_STATS.nodes);
            print!(" nps {}", SEARCH_STATS.nps());
        }
        print!(" pv ");

        // Output best move
        let mut move_str = String::new();
        move_str.push(( (best_move.from % 8) as u8 + b'a') as char);
        move_str.push(( (best_move.from / 8) as u8 + b'1') as char);
        move_str.push(( (best_move.to % 8) as u8 + b'a') as char);
        move_str.push(( (best_move.to / 8) as u8 + b'1') as char);
        if best_move.promo != 0 {
            match best_move.promo {
                x if x == QUEEN as i32 => move_str.push('q'),
                x if x == ROOK as i32 => move_str.push('r'),
                x if x == BISHOP as i32 => move_str.push('b'),
                x if x == KNIGHT as i32 => move_str.push('n'),
                _ => {}
            }
        }
        println!(" {}", move_str);

        if score.abs() >= MATE - 1000 {
            break;
        }
    }

    score
}

fn init_tables() {
    for sq in 0..64 {
        let x = sq % 8;
        let y = sq / 8;
        let mut km: U64 = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    km |= 1u64 << ((ny * 8 + nx) as usize);
                }
            }
        }
        unsafe { KING_MOVES[sq] = km; }

        let mut nm: U64 = 0;
        let kdx = [2, 2, -2, -2, 1, 1, -1, -1];
        let kdy = [1, -1, 1, -1, 2, -2, 2, -2];
        for i in 0..8 {
            let nx = x as i32 + kdx[i];
            let ny = y as i32 + kdy[i];
            if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                nm |= 1u64 << ((ny * 8 + nx) as usize);
            }
        }
        unsafe { KNIGHT_MOVES[sq] = nm; }
    }

    init_zobrist();
    unsafe {
        HISTORY_TABLE.init();
        KILLER_MOVES.init();
        for i in 0..TT_SIZE {
            TRANSPOSITION_TABLE[i] = TTEntry::new();
        }
    }
}

fn parse_move(b: &Board, move_str: &str, parsed_move: &mut Move) -> bool {
    let moves = generate_moves(b, false);
    if move_str.len() < 4 {
        return false;
    }
    let from = (move_str.as_bytes()[0] - b'a') as usize + ((move_str.as_bytes()[1] - b'1') as usize) * 8;
    let to = (move_str.as_bytes()[2] - b'a') as usize + ((move_str.as_bytes()[3] - b'1') as usize) * 8;
    let mut promo_piece = 0;
    if move_str.len() == 5 {
        match move_str.as_bytes()[4] as char {
            'q' => promo_piece = QUEEN as i32,
            'r' => promo_piece = ROOK as i32,
            'b' => promo_piece = BISHOP as i32,
            'n' => promo_piece = KNIGHT as i32,
            _ => promo_piece = 0,
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

fn main() {
    init_tables();
    let mut board = Board::new();
    board.init();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if line.is_err() { break; }
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => continue,
        };

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
                if let Some(on) = parts.next() {
                    option_name.push_str(on);
                    while let Some(next_token) = parts.next() {
                        if next_token == "value" {
                            break;
                        } else {
                            option_name.push_str(next_token);
                        }
                    }
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
                        // Could resize TT here if needed
                    }
                }
            }
        } else if cmd == "isready" {
            println!("readyok");
        } else if cmd == "ucinewgame" {
            board.init();
            unsafe {
                HISTORY_TABLE.init();
                KILLER_MOVES.init();
                for i in 0..TT_SIZE {
                    TRANSPOSITION_TABLE[i] = TTEntry::new();
                }
            }
        } else if cmd == "position" {
            let sub_cmd = parts.next().unwrap_or("");
            let mut token = "";
            if sub_cmd == "startpos" {
                board.init();
                token = parts.next().unwrap_or("");
            } else if sub_cmd == "fen" {
                board.init();
                // consume until "moves" or end
                while let Some(t) = parts.next() {
                    if t == "moves" {
                        token = "moves";
                        break;
                    }
                }
            }

            if token == "moves" {
                let mut m = Move::default();
                while let Some(tok) = parts.next() {
                    if parse_move(&board, tok, &mut m) {
                        make_move(&mut board, m);
                    }
                }
            }
        } else if cmd == "go" {
            unsafe {
                // default
            }
            let mut search_depth: i32 = unsafe { UCI_OPTIONS.depth };
            let mut move_time: i32 = 0;
            let mut wtime: i32 = 0;
            let mut btime: i32 = 0;
            let mut winc: i32 = 0;
            let mut binc: i32 = 0;
            let mut movestogo: i32 = 40;
            let mut infinite = false;

            let mut token_iter = parts.peekable();
            while let Some(token) = token_iter.next() {
                match token {
                    "depth" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                search_depth = max(1, min(30, v));
                            }
                        }
                    }
                    "movetime" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                move_time = v;
                            }
                        }
                    }
                    "wtime" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                wtime = v;
                            }
                        }
                    }
                    "btime" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                btime = v;
                            }
                        }
                    }
                    "winc" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                winc = v;
                            }
                        }
                    }
                    "binc" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                binc = v;
                            }
                        }
                    }
                    "movestogo" => {
                        if let Some(s) = token_iter.next() {
                            if let Ok(v) = s.parse::<i32>() {
                                movestogo = v;
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

            let allocated_time: i32;
            if !infinite && move_time == 0 && (wtime > 0 || btime > 0) {
                let time_left = if board.side == WHITE { wtime } else { btime };
                let increment = if board.side == WHITE { winc } else { binc };
                let mut at = (time_left / movestogo) + ( (increment as f32 * 0.8) as i32 );
                at = min(at, time_left / 3);
                allocated_time = at;
            } else if move_time > 0 {
                allocated_time = (move_time as f32 * 0.95) as i32;
            } else {
                allocated_time = 0;
            }

            let mut best_move = Move::default();
            iterative_deepening(&board, search_depth, &mut best_move, allocated_time);

            if best_move.from != best_move.to || best_move.from != 0 {
                let mut move_str = String::new();
                move_str.push(((best_move.from % 8) as u8 + b'a') as char);
                move_str.push(((best_move.from / 8) as u8 + b'1') as char);
                move_str.push(((best_move.to % 8) as u8 + b'a') as char);
                move_str.push(((best_move.to / 8) as u8 + b'1') as char);
                if best_move.promo != 0 {
                    match best_move.promo {
                        x if x == QUEEN as i32 => move_str.push('q'),
                        x if x == ROOK as i32 => move_str.push('r'),
                        x if x == BISHOP as i32 => move_str.push('b'),
                        x if x == KNIGHT as i32 => move_str.push('n'),
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
                            x if x == QUEEN as i32 => move_str.push('q'),
                            x if x == ROOK as i32 => move_str.push('r'),
                            x if x == BISHOP as i32 => move_str.push('b'),
                            x if x == KNIGHT as i32 => move_str.push('n'),
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