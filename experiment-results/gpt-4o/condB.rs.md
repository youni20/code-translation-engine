use std::time::Instant;
use std::sync::OnceLock;
use std::convert::TryInto;

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

static mut KING_MOVES: [U64; 64] = [0; 64];
static mut KNIGHT_MOVES: [U64; 64] = [0; 64];

struct HistoryTable {
    scores: [[[i32; 64]; 64]; 2], // [side][from][to]
}

impl HistoryTable {
    fn new() -> Self {
        HistoryTable {
            scores: [[[0; 64]; 64]; 2],
        }
    }

    fn update(&mut self, side: usize, from: usize, to: usize, depth: i32) {
        self.scores[side][from][to] += depth * depth;
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

static HISTORY_TABLE: OnceLock<HistoryTable> = OnceLock::new();

struct KillerMoves {
    killers: [[Option<Move>; 2]; MAX_PLY],
}

impl KillerMoves {
    fn new() -> Self {
        KillerMoves {
            killers: [[None, None]; MAX_PLY],
        }
    }

    fn update(&mut self, m: Move, ply: usize) {
        if self.killers[ply][0] != Some(m) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(m);
        }
    }

    fn is_killer(&self, m: Move, ply: usize) -> bool {
        self.killers[ply][0] == Some(m) || self.killers[ply][1] == Some(m)
    }
}

static KILLER_MOVES: OnceLock<KillerMoves> = OnceLock::new();

#[derive(Copy, Clone)]
struct TTEntry {
    hash: U64,
    depth: i32,
    score: i32,
    flag: i32,
    best_move: i32,
}

const TT_SIZE: usize = 1 << 20;
static mut TRANSPOSITION_TABLE: [TTEntry; TT_SIZE] = unsafe { std::mem::transmute([0u8; TT_SIZE * std::mem::size_of::<TTEntry>()]) };

const TT_EXACT: i32 = 0;
const TT_ALPHA: i32 = 1;
const TT_BETA: i32 = 2;

struct UCIOptions {
    depth: i32,
    use_quiescence: bool,
    quiescence_depth: i32,
}

impl UCIOptions {
    fn new() -> Self {
        UCIOptions {
            depth: 8,
            use_quiescence: true,
            quiescence_depth: 4,
        }
    }
}

static UCI_OPTIONS: OnceLock<UCIOptions> = OnceLock::new();

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

static SEARCH_STATS: OnceLock<SearchStats> = OnceLock::new();

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
        self.pieces[WHITE][PAWN] = 0xFF00;
        self.pieces[WHITE][KNIGHT] = 0x42;
        self.pieces[WHITE][BISHOP] = 0x24;
        self.pieces[WHITE][ROOK] = 0x81;
        self.pieces[WHITE][QUEEN] = 0x8;
        self.pieces[WHITE][KING] = 0x10;

        self.pieces[BLACK][PAWN] = 0xFF000000000000;
        self.pieces[BLACK][KNIGHT] = 0x4200000000000000;
        self.pieces[BLACK][BISHOP] = 0x2400000000000000;
        self.pieces[BLACK][ROOK] = 0x8100000000000000;
        self.pieces[BLACK][QUEEN] = 0x800000000000000;
        self.pieces[BLACK][KING] = 0x1000000000000000;

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
        let mut eval = 0;
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
            let king_sq = self.pieces[c][KING].trailing_zeros() as i32;

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

        let center = 0x0000001818000000;
        eval += (self.pieces[WHITE][PAWN] & center).count_ones() as i32 * 20;
        eval -= (self.pieces[BLACK][PAWN] & center).count_ones() as i32 * 20;

        let mut passed_pawns_white = self.pieces[WHITE][PAWN];
        while passed_pawns_white != 0 {
            let sq = passed_pawns_white.trailing_zeros() as i32;
            let rank = sq / 8;
            if rank >= 4 {
                eval += (rank - 3) * 15;
            }
            passed_pawns_white &= passed_pawns_white - 1;
        }

        let mut passed_pawns_black = self.pieces[BLACK][PAWN];
        while passed_pawns_black != 0 {
            let sq = passed_pawns_black.trailing_zeros() as i32;
            let rank = sq / 8;
            if rank <= 3 {
                eval -= (4 - rank) * 15;
            }
            passed_pawns_black &= passed_pawns_black - 1;
        }

        if self.side == WHITE {
            eval
        } else {
            -eval
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Move {
    from: usize,
    to: usize,
    score: i32,
    piece: i32,
    captured: i32,
    promo: i32,
}

impl Move {
    fn new(f: usize, t: usize, p: i32, c: i32, pr: i32) -> Self {
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

static mut ZOBRIST_PIECES: [[[U64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
static mut ZOBRIST_CASTLE: [U64; 16] = [0; 16];
static mut ZOBRIST_EP: [U64; 64] = [0; 64];
static mut ZOBRIST_SIDE: U64 = 0;

fn init_zobrist() {
    for c in 0..2 {
        for p in 0..6 {
            for sq in 0..64 {
                unsafe {
                    ZOBRIST_PIECES[c][p][sq] = rand_u64();
                }
            }
        }
    }

    for i in 0..16 {
        unsafe {
            ZOBRIST_CASTLE[i] = rand_u64();
        }
    }

    for i in 0..64 {
        unsafe {
            ZOBRIST_EP[i] = rand_u64();
        }
    }

    unsafe {
        ZOBRIST_SIDE = rand_u64();
    }
}

fn rand_u64() -> U64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    (0..64).fold(0u64, |acc, _| (acc << 1) | ((nanos % 2 == 0) as U64))
}

fn zobrist_hash(b: &Board) -> U64 {
    let mut hash = 0;

    for c in 0..2 {
        for p in 0..6 {
            let mut bb = b.pieces[c][p];
            while bb != 0 {
                let sq = bb.trailing_zeros();
                hash ^= unsafe { ZOBRIST_PIECES[c][p][sq as usize] };
                bb &= bb - 1;
            }
        }
    }

    hash ^= unsafe { ZOBRIST_CASTLE[b.castle as usize] };
    if b.ep != -1 {
        hash ^= unsafe { ZOBRIST_EP[b.ep as usize] };
    }
    if b.side == BLACK {
        hash ^= unsafe { ZOBRIST_SIDE };
    }

    hash
}

fn get_rook_attacks(sq: i32, blockers: U64) -> U64 {
    let mut attacks = 0;
    let tr = sq / 8;
    let tf = sq % 8;

    for r in tr + 1..=7 {
        attacks |= 1 << (r * 8 + tf);
        if (1 << (r * 8 + tf)) & blockers != 0 {
            break;
        }
    }
    for r in (0..tr).rev() {
        attacks |= 1 << (r * 8 + tf);
        if (1 << (r * 8 + tf)) & blockers != 0 {
            break;
        }
    }
    for f in tf + 1..=7 {
        attacks |= 1 << (tr * 8 + f);
        if (1 << (tr * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for f in (0..tf).rev() {
        attacks |= 1 << (tr * 8 + f);
        if (1 << (tr * 8 + f)) & blockers != 0 {
            break;
        }
    }
    attacks
}

fn get_bishop_attacks(sq: i32, blockers: U64) -> U64 {
    let mut attacks = 0;
    let tr = sq / 8;
    let tf = sq % 8;

    for (r, f) in (tr + 1..=7).zip(tf + 1..=7) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for (r, f) in (tr + 1..=7).zip((0..tf).rev()) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for (r, f) in (0..tr).rev().zip(tf + 1..=7) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for (r, f) in (0..tr).rev().zip((0..tf).rev()) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    attacks
}

fn is_attacked(sq: i32, attacker: usize, b: &Board) -> bool {
    if attacker == WHITE {
        if (sq >= 9) && (sq % 8 != 0) && ((1 << (sq - 9)) & b.pieces[WHITE][PAWN] != 0) {
            return true;
        }
        if (sq >= 7) && (sq % 8 != 7) && ((1 << (sq - 7)) & b.pieces[WHITE][PAWN] != 0) {
            return true;
        }
    } else {
        if (sq <= 56) && (sq % 8 != 0) && ((1 << (sq + 7)) & b.pieces[BLACK][PAWN] != 0) {
            return true;
        }
        if (sq <= 54) && (sq % 8 != 7) && ((1 << (sq + 9)) & b.pieces[BLACK][PAWN] != 0) {
            return true;
        }
    }

    unsafe {
        if KING_MOVES[sq as usize] & b.pieces[attacker][KNIGHT] != 0 {
            return true;
        }
        if KING_MOVES[sq as usize] & b.pieces[attacker][KING] != 0 {
            return true;
        }
    }

    if get_rook_attacks(sq, b.all)
        & (b.pieces[attacker][ROOK] | b.pieces[attacker][QUEEN])
        != 0
    {
        return true;
    }
    if get_bishop_attacks(sq, b.all)
        & (b.pieces[attacker][BISHOP] | b.pieces[attacker][QUEEN])
        != 0
    {
        return true;
    }

    false
}

fn is_in_check(b: &Board) -> bool {
    if b.pieces[b.side][KING] == 0 {
        return false;
    }
    let king_sq = b.pieces[b.side][KING].trailing_zeros() as i32;
    is_attacked(king_sq, 1 - b.side, b)
}

fn make_move(b: &mut Board, m: &Move) {
    let from_bb = 1 << m.from;
    let to_bb = 1 << m.to;

    let opponent = 1 - b.side;
    let prev_ep = b.ep;

    b.hash ^= unsafe { ZOBRIST_PIECES[b.side][m.piece as usize][m.from] };
    b.hash ^= unsafe { ZOBRIST_PIECES[b.side][m.piece as usize][m.to] };

    if b.ep != -1 {
        b.hash ^= unsafe { ZOBRIST_EP[b.ep as usize] };
    }
    b.hash ^= unsafe { ZOBRIST_CASTLE[b.castle as usize] };

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

    b.hash ^= unsafe { ZOBRIST_CASTLE[b.castle as usize] };
    b.ep = -1;

    b.pieces[b.side][m.piece as usize] ^= from_bb | to_bb;

    for p in PAWN..=KING {
        if b.pieces[opponent][p] & to_bb != 0 {
            b.pieces[opponent][p] ^= to_bb;
            b.hash ^= unsafe { ZOBRIST_PIECES[opponent][p][m.to] };
            break;
        }
    }

    if m.piece == PAWN {
        if m.to as i32 == prev_ep {
            let captured_pawn_sq = if b.side == WHITE {
                m.to as i32 - 8
            } else {
                m.to as i32 + 8
            };
            b.pieces[opponent][PAWN] ^= 1 << captured_pawn_sq;
            b.hash ^= unsafe { ZOBRIST_PIECES[opponent][PAWN][captured_pawn_sq as usize] };
        }
        if (m.from as i32 - m.to as i32).abs() == 16 {
            b.ep = if b.side == WHITE {
                m.from as i32 + 8
            } else {
                m.from as i32 - 8
            };
            b.hash ^= unsafe { ZOBRIST_EP[b.ep as usize] };
        }
        if m.promo != 0 {
            b.pieces[b.side][PAWN] ^= to_bb;
            b.pieces[b.side][m.promo as usize] ^= to_bb;
            b.hash ^= unsafe { ZOBRIST_PIECES[b.side][PAWN][m.to] };
            b.hash ^= unsafe { ZOBRIST_PIECES[b.side][m.promo as usize][m.to] };
        }
    } else if m.piece == KING {
        if (m.from as i32 - m.to as i32).abs() == 2 {
            if m.to == 6 {
                b.pieces[WHITE][ROOK] ^= (1 << 7) | (1 << 5);
                b.hash ^= unsafe { ZOBRIST_PIECES[WHITE][ROOK][7] };
                b.hash ^= unsafe { ZOBRIST_PIECES[WHITE][ROOK][5] };
            } else if m.to == 2 {
                b.pieces[WHITE][ROOK] ^= (1 << 0) | (1 << 3);
                b.hash ^= unsafe { ZOBRIST_PIECES[WHITE][ROOK][0] };
                b.hash ^= unsafe { ZOBRIST_PIECES[WHITE][ROOK][3] };
            } else if m.to == 62 {
                b.pieces[BLACK][ROOK] ^= (1 << 63) | (1 << 61);
                b.hash ^= unsafe { ZOBRIST_PIECES[BLACK][ROOK][63] };
                b.hash ^= unsafe { ZOBRIST_PIECES[BLACK][ROOK][61] };
            } else if m.to == 58 {
                b.pieces[BLACK][ROOK] ^= (1 << 56) | (1 << 59);
                b.hash ^= unsafe { ZOBRIST_PIECES[BLACK][ROOK][56] };
                b.hash ^= unsafe { ZOBRIST_PIECES[BLACK][ROOK][59] };
            }
        }
    }

    b.update();
    b.side = opponent;
    b.hash ^= unsafe { ZOBRIST_SIDE };
}

fn is_legal_move(b: &mut Board, m: &Move) -> bool {
    let mut copy = b_new(b);
    make_move(&mut copy, m);
    if copy.pieces[b.side][KING] == 0 {
        return false;
    }
    let king_sq = copy.pieces[b.side][KING].trailing_zeros() as i32;
    !is_attacked(king_sq, copy.side, &copy)
}

fn b_new(b: &Board) -> Board {
    let mut b_clone = Board::new();
    b_clone.pieces = b.pieces;
    b_clone.occupied = b.occupied;
    b_clone.all = b.all;
    b_clone.side = b.side;
    b_clone.ep = b.ep;
    b_clone.castle = b.castle;
    b_clone.hash = b.hash;
    b_clone
}

fn generate_moves(b: &Board, captures_only: bool) -> Vec<Move> {
    let mut moves = Vec::with_capacity(if captures_only { 32 } else { 128 });

    for p in PAWN..=KING {
        let mut bitboard = b.pieces[b.side][p];
        while bitboard != 0 {
            let from = bitboard.trailing_zeros() as usize;
            let mut attacks = 0;

            if p == PAWN {
                let dir = if b.side == WHITE { 8 } else { -8 };
                let promo_rank = if b.side == WHITE { 7 } else { 0 };

                if !captures_only {
                    let to_sq = from as i32 + dir;
                    if (0..64).contains(&to_sq) && (b.all & (1 << to_sq)) == 0 {
                        if (to_sq / 8) == promo_rank {
                            moves.push(Move::new(from, to_sq as usize, p as i32, -1, QUEEN as i32));
                        } else {
                            moves.push(Move::new(from, to_sq as usize, p as i32, -1, 0));
                            let start_rank = if b.side == WHITE { 1 } else { 6 };
                            if (from as i32 / 8) == start_rank {
                                let to_sq2 = from as i32 + 2 * dir;
                                if (b.all & (1 << to_sq2)) == 0 {
                                    moves.push(Move::new(from, to_sq2 as usize, p as i32, -1, 0));
                                }
                            }
                        }
                    }
                }

                let cap_dirs = [dir - 1, dir + 1];
                for d in cap_dirs.iter() {
                    let to = from as i32 + d;
                    if to < 0 || to > 63 || ((from as i32 % 8) - (to % 8)).abs() > 1 {
                        continue;
                    }

                    if (b.occupied[1 - b.side] & (1 << to)) != 0 {
                        if (to / 8) == promo_rank {
                            moves.push(Move::new(from, to as usize, p as i32, 0, QUEEN as i32));
                        } else {
                            moves.push(Move::new(from, to as usize, p as i32, 0, 0));
                        }
                    } else if !captures_only && to == b.ep {
                        moves.push(Move::new(from, to as usize, p as i32, 0, 0));
                    }
                }
            } else if p == KING && !captures_only {
                attacks = unsafe { KING_MOVES[from] & !b.occupied[b.side] };

                if !is_in_check(b) {
                    if b.side == WHITE {
                        if (b.castle & 1) != 0 && (b.all & 0x60) == 0 {
                            if !is_attacked(5, BLACK, b) && !is_attacked(6, BLACK, b) {
                                moves.push(Move::new(4, 6, KING as i32, 0, 0));
                            }
                        }
                        if (b.castle & 2) != 0 && (b.all & 0xE) == 0 {
                            if !is_attacked(3, BLACK, b) && !is_attacked(2, BLACK, b) {
                                moves.push(Move::new(4, 2, KING as i32, 0, 0));
                            }
                        }
                    } else {
                        if (b.castle & 4) != 0 && (b.all & 0x6000000000000000) == 0 {
                            if !is_attacked(61, WHITE, b) && !is_attacked(62, WHITE, b) {
                                moves.push(Move::new(60, 62, KING as i32, 0, 0));
                            }
                        }
                        if (b.castle & 8) != 0 && (b.all & 0xE00000000000000) == 0 {
                            if !is_attacked(59, WHITE, b) && !is_attacked(58, WHITE, b) {
                                moves.push(Move::new(60, 58, KING as i32, 0, 0));
                            }
                        }
                    }
                }
            } else {
                if p == KNIGHT {
                    attacks = unsafe { KNIGHT_MOVES[from] };
                } else if p == BISHOP {
                    attacks = get_bishop_attacks(from as i32, b.all);
                } else if p == ROOK {
                    attacks = get_rook_attacks(from as i32, b.all);
                } else if p == QUEEN {
                    attacks = get_rook_attacks(from as i32, b.all) | get_bishop_attacks(from as i32, b.all);
                } else if p == KING {
                    attacks = unsafe { KING_MOVES[from] };
                }

                if captures_only {
                    attacks &= b.occupied[1 - b.side];
                } else {
                    attacks &= !b.occupied[b.side];
                }
            }

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                moves.push(Move::new(from, to, p as i32, 0, 0));
                attacks &= attacks - 1;
            }

            bitboard &= bitboard - 1;
        }
    }

    moves.into_iter().filter(|m| is_legal_move(&mut b_new(b), m)).collect()
}

fn score_moves(moves: &mut [Move], b: &Board, tt_move: Option<&Move>, ply: usize) {
    let killer_moves = KILLER_MOVES.get().unwrap();
    let history_table = HISTORY_TABLE.get().unwrap();

    for m in moves.iter_mut() {
        if let Some(tt) = tt_move {
            if *m == *tt {
                m.score = 1_000_000;
                continue;
            }
        }

        if (b.occupied[1 - b.side] & (1 << m.to)) != 0 {
            let victim_values = [100, 300, 300, 500, 900, 10_000];
            for p in (PAWN..=KING).rev() {
                if (b.pieces[1 - b.side][p] & (1 << m.to)) != 0 {
                    m.score = 100_000 + victim_values[p] * 10 - victim_values[m.piece as usize];
                    break;
                }
            }
        } else if killer_moves.is_killer(*m, ply) {
            m.score = 90_000;
        } else {
            m.score = history_table.get(b.side, m.from, m.to);
        }

        if m.promo == QUEEN as i32 {
            m.score += 80_000;
        }
    }

    moves.sort_unstable_by(|a, b| b.score.cmp(&a.score));
}

fn quiescence(b: &mut Board, mut alpha: i32, beta: i32, depth: i32) -> i32 {
    SEARCH_STATS.get().unwrap().qnodes += 1;

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
        let gain = if m.piece != PAWN { 900 } else { 200 };
        if stand_pat + gain < alpha && depth < -1 {
            continue;
        }

        let mut copy = b_new(b);
        make_move(&mut copy, m);

        let score = -quiescence(&mut copy, -beta, -alpha, depth - 1);

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
    b: &mut Board,
    mut depth: i32,
    mut alpha: i32,
    beta: i32,
    best_move: &mut Move,
    ply: usize,
    null_move: bool,
) -> i32 {
    SEARCH_STATS.get().unwrap().nodes += 1;

    let in_check = is_in_check(b);
    if in_check {
        depth += 1;
    }

    let killer_moves = KILLER_MOVES.get().unwrap();
    let history_table = HISTORY_TABLE.get().unwrap();

    let tt_index = (b.hash % TT_SIZE as u64) as usize;
    let tt_entry = unsafe { &mut TRANSPOSITION_TABLE[tt_index] };
    let mut tt_move = None;

    if tt_entry.hash == b.hash && tt_entry.depth >= depth {
        if tt_entry.flag == TT_EXACT {
            if ply == 0 {
                best_move.from = tt_entry.best_move as usize & 63;
                best_move.to = (tt_entry.best_move >> 6) as usize & 63;
                best_move.piece = (tt_entry.best_move >> 12) as i32 & 7;
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
        tt_move = Some(Move::new(
            tt_entry.best_move as usize & 63,
            (tt_entry.best_move >> 6) as usize & 63,
            (tt_entry.best_move >> 12) as i32 & 7,
            -1,
            0,
        ));
    }

    if depth <= 0 {
        return quiescence(b, alpha, beta, 0);
    }

    if null_move && !in_check && depth >= 3 && ply > 0 {
        let mut copy = b_new(b);
        copy.side = 1 - copy.side;
        copy.hash ^= unsafe { ZOBRIST_SIDE };
        copy.ep = -1;

        let score = -search(&mut copy, depth - 1 - if depth > 6 { 3 } else { 2 }, -beta, -beta + 1, best_move, ply + 1, false);

        if score >= beta {
            return beta;
        }
    }

    let mut moves = generate_moves(b, false);
    if moves.is_empty() {
        return if in_check { -MATE + ply as i32 } else { 0 };
    }

    score_moves(&mut moves, b, tt_move.as_ref(), ply);
    *best_move = moves[0];

    let mut best_score = -INF;
    let orig_alpha = alpha;

    for (i, m) in moves.iter().enumerate() {
        let mut reduction = 0;
        if i > 4
            && depth >= 3
            && !in_check
            && (b.occupied[1 - b.side] & (1 << m.to)) == 0
            && m.promo == 0
        {
            reduction = if i > 12 { 3 } else if i > 6 { 2 } else { 1 };

            if killer_moves.is_killer(*m, ply) || history_table.get(b.side, m.from, m.to) > 5000 {
                reduction = std::cmp::max(0, reduction - 1);
            }
        }

        let mut copy = b_new(b);
        make_move(&mut copy, m);

        let score = if i == 0 {
            -search(&mut copy, depth - 1 - reduction, -beta, -alpha, best_move, ply + 1, true)
        } else {
            let score = -search(&mut copy, depth - 1 - reduction, -alpha - 1, -alpha, best_move, ply + 1, true);

            if score > alpha && score < beta {
                -search(&mut copy, depth - 1, -beta, -alpha, best_move, ply + 1, true)
            } else {
                score
            }
        };

        if reduction > 0 && score > alpha {
            let score = -search(&mut copy, depth - 1, -beta, -alpha, best_move, ply + 1, true);
            alpha = std::cmp::max(alpha, score);
        }

        if score > best_score {
            best_score = score;
            *best_move = *m;
        }

        if score > alpha {
            alpha = score;
            if (b.occupied[1 - b.side] & (1 << m.to)) == 0 {
                history_table.update(b.side, m.from, m.to, depth);
            }
        }

        if alpha >= beta {
            if (b.occupied[1 - b.side] & (1 << m.to)) == 0 {
                killer_moves.update(*m, ply);
            }
            break;
        }

        if depth <= 2
            && !in_check
            && i > 8
            && (b.occupied[1 - b.side] & (1 << m.to)) == 0
            && b.evaluate() + depth * 100 < alpha
        {
            break;
        }
    }

    tt_entry.hash = b.hash;
    tt_entry.depth = depth;
    tt_entry.score = best_score;
    tt_entry.best_move = (best_move.from | (best_move.to << 6) | (best_move.piece << 12)) as i32;
    tt_entry.flag = if best_score <= orig_alpha {
        TT_ALPHA
    } else if best_score >= beta {
        TT_BETA
    } else {
        TT_EXACT
    };

    best_score
}

fn iterative_deepening(b: &mut Board, max_depth: i32, best_move: &mut Move, time_limit: i32) -> i32 {
    let mut score = 0;
    let mut alpha = -INF;
    let mut beta = INF;
    let mut window = 50;

    SEARCH_STATS.get().unwrap().init();

    for depth in 1..=max_depth {
        SEARCH_STATS.get().unwrap().current_depth = depth;

        if depth >= 4 {
            alpha = score - window;
            beta = score + window;
        }

        let mut temp_score = search(b, depth, alpha, beta, best_move, 0, true);

        if temp_score <= alpha || temp_score >= beta {
            temp_score = search(b, depth, -INF, INF, best_move, 0, true);
            window = 50;
        } else {
            window = 25;
        }

        score = temp_score;

        if time_limit > 0 {
            let ms = SEARCH_STATS.get().unwrap().start_time.elapsed().as_millis() as i32;

            if ms > time_limit * 4 / 10 && depth > 4 {
                break;
            }
        }

        print!(
            "info depth {} score cp {} nodes {} nps {} pv {}",
            depth,
            score,
            SEARCH_STATS.get().unwrap().nodes,
            SEARCH_STATS.get().unwrap().nps(),
            format_move(best_move)
        );

        if score.abs() >= MATE - 1000 {
            break;
        }
    }

    score
}

fn format_move(m: &Move) -> String {
    let mut move_str = String::new();
    move_str.push((m.from % 8 + b'a') as char);
    move_str.push((m.from / 8 + b'1') as char);
    move_str.push((m.to % 8 + b'a') as char);
    move_str.push((m.to / 8 + b'1') as char);
    if m.promo != 0 {
        match m.promo {
            QUEEN => move_str.push('q'),
            ROOK => move_str.push('r'),
            BISHOP => move_str.push('b'),
            KNIGHT => move_str.push('n'),
            _ => {}
        }
    }
    move_str
}

fn init_tables() {
    unsafe {
        for sq in 0..64 {
            let x = sq % 8;
            let y = sq / 8;

            KING_MOVES[sq] = 0;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                        KING_MOVES[sq] |= 1 << (ny * 8 + nx) as u64;
                    }
                }
            }

            KNIGHT_MOVES[sq] = 0;
            let kdx = [2, 2, -2, -2, 1, 1, -1, -1];
            let kdy = [1, -1, 1, -1, 2, -2, 2, -2];
            for i in 0..8 {
                let nx = x as i32 + kdx[i];
                let ny = y as i32 + kdy[i];
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    KNIGHT_MOVES[sq] |= 1 << (ny * 8 + nx) as u64;
                }
            }
        }
    }

    init_zobrist();
    HISTORY_TABLE.set(HistoryTable::new()).unwrap();
    KILLER_MOVES.set(KillerMoves::new()).unwrap();
}

fn parse_move(b: &mut Board, move_str: &str) -> Option<Move> {
    let moves = generate_moves(b, false);
    let from = ((move_str.as_bytes()[0] - b'a') + (move_str.as_bytes()[1] - b'1') * 8) as usize;
    let to = ((move_str.as_bytes()[2] - b'a') + (move_str.as_bytes()[3] - b'1') * 8) as usize;
    let promo_piece = if move_str.len() == 5 {
        match move_str.as_bytes()[4] {
            b'q' => QUEEN,
            b'r' => ROOK,
            b'b' => BISHOP,
            b'n' => KNIGHT,
            _ => 0,
        }
    } else {
        0
    };

    for m in moves {
        if m.from == from && m.to == to && (m.promo == promo_piece || m.promo == 0) {
            return Some(m);
        }
    }
    None
}

fn main() {
    init_tables();
    UCI_OPTIONS.set(UCIOptions::new()).unwrap();
    SEARCH_STATS.set(SearchStats::new()).unwrap();

    let mut board = Board::new();
    board.init();

    let mut input = String::new();

    while let Ok(_) = std::io::stdin().read_line(&mut input) {
        let line = input.trim();
        let mut parts = line.split_whitespace();

        if let Some(cmd) = parts.next() {
            match cmd {
                "uci" => {
                    println!("id name NanoChessTurbo");
                    println!("id author CrvProject");
                    println!("option name Depth type spin default 10 min 1 max 30");
                    println!("option name Hash type spin default 64 min 1 max 1024");
                    println!("uciok");
                },
                "setoption" => {
                    if let Some("name") = parts.next() {
                        if let Some(option_name) = parts.next() {
                            if let Some(next_token) = parts.next() {
                                if next_token == "value" {
                                    if option_name == "Depth" {
                                        if let Some(value_str) = parts.next() {
                                            if let Ok(value) = value_str.parse() {
                                                UCI_OPTIONS.get().unwrap().depth = value.clamp(1, 30);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "isready" => {
                    println!("readyok");
                },
                "ucinewgame" => {
                    board.init();
                    HISTORY_TABLE.set(HistoryTable::new()).unwrap();
                    KILLER_MOVES.set(KillerMoves::new()).unwrap();
                },
                "position" => {
                    if let Some(sub_cmd) = parts.next() {
                        if sub_cmd == "startpos" {
                            board.init();
                        } else if sub_cmd == "fen" {
                            board.init();
                        }

                        if let Some("moves") = parts.next() {
                            for move_str in parts {
                                if let Some(parsed_move) = parse_move(&mut board, move_str) {
                                    make_move(&mut board, &parsed_move);
                                }
                            }
                        }
                    }
                },
                "go" => {
                    let mut search_depth = UCI_OPTIONS.get().unwrap().depth;
                    let mut move_time = 0;
                    let mut wtime = 0;
                    let mut btime = 0;
                    let mut winc = 0;
                    let mut binc = 0;
                    let mut movestogo = 40;
                    let mut infinite = false;

                    while let Some(token) = parts.next() {
                        match token {
                            "depth" => {
                                if let Some(value_str) = parts.next() {
                                    if let Ok(value) = value_str.parse() {
                                        search_depth = value.clamp(1, 30);
                                    }
                                }
                            },
                            "movetime" => {
                                if let Some(value_str) = parts.next() {
                                    move_time = value_str.parse().unwrap_or(0);
                                }
                            },
                            "wtime" => {
                                if let Some(value_str) = parts.next() {
                                    wtime = value_str.parse().unwrap_or(0);
                                }
                            },
                            "btime" => {
                                if let Some(value_str) = parts.next() {
                                    btime = value_str.parse().unwrap_or(0);
                                }
                            },
                            "winc" => {
                                if let Some(value_str) = parts.next() {
                                    winc = value_str.parse().unwrap_or(0);
                                }
                            },
                            "binc" => {
                                if let Some(value_str) = parts.next() {
                                    binc = value_str.parse().unwrap_or(0);
                                }
                            },
                            "movestogo" => {
                                if let Some(value_str) = parts.next() {
                                    movestogo = value_str.parse().unwrap_or(40);
                                }
                            },
                            "infinite" => {
                                infinite = true;
                                search_depth = 20;
                            },
                            _ => {}
                        }
                    }

                    let allocated_time = if !infinite && move_time == 0 && (wtime > 0 || btime > 0) {
                        let time_left = if board.side == WHITE { wtime } else { btime };
                        let increment = if board.side == WHITE { winc } else { binc };

                        (time_left / movestogo + (increment as f64 * 0.8) as i32).min(time_left / 3)
                    } else if move_time > 0 {
                        (move_time as f64 * 0.95) as i32
                    } else {
                        0
                    };

                    let mut best_move = Move::new(0, 0, -1, -1, 0);
                    iterative_deepening(&mut board, search_depth, &mut best_move, allocated_time);

                    println!("bestmove {}", format_move(&best_move));
                },
                "quit" => {
                    break;
                },
                _ => {}
            }
        }

        input.clear();
    }
}