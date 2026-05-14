use std::time::{Duration, Instant};
use std::sync::LazyLock;

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
    scores: [[[i32; 64]; 64]; 2],
}

impl HistoryTable {
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

static mut HISTORY_TABLE: LazyLock<HistoryTable> = LazyLock::new(|| HistoryTable {
    scores: [[[0; 64]; 64]; 2],
});

struct KillerMoves {
    killers: [[Option<*const Move>; 2]; MAX_PLY],
}

impl KillerMoves {
    fn init(&mut self) {
        for ply in 0..MAX_PLY {
            self.killers[ply] = [None, None];
        }
    }

    fn update(&mut self, m: *const Move, ply: usize) {
        if self.killers[ply][0] != Some(m) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(m);
        }
    }

    fn is_killer(&self, m: *const Move, ply: usize) -> bool {
        self.killers[ply][0] == Some(m) || self.killers[ply][1] == Some(m)
    }
}

static mut KILLER_MOVES: LazyLock<KillerMoves> = LazyLock::new(|| KillerMoves {
    killers: [[None; 2]; MAX_PLY],
});

#[derive(Default, Copy, Clone)]
struct TTEntry {
    hash: U64,
    depth: i32,
    score: i32,
    flag: i32,
    best_move: Option<Move>,
}

static mut TRANSPOSITION_TABLE: LazyLock<[TTEntry; TT_SIZE]> = LazyLock::new(|| [TTEntry {
    hash: 0,
    depth: 0,
    score: 0,
    flag: 0,
    best_move: None,
}; TT_SIZE]);

const TT_SIZE: usize = 1 << 20;
const TT_EXACT: i32 = 0;
const TT_ALPHA: i32 = 1;
const TT_BETA: i32 = 2;

struct UCIOptions {
    depth: usize,
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

static mut UCI_OPTIONS: LazyLock<UCIOptions> = LazyLock::new(|| UCIOptions {
    depth: 8,
    use_quiescence: true,
    quiescence_depth: 4,
});

struct SearchStats {
    nodes: i64,
    qnodes: i64,
    current_depth: i32,
    start_time: Instant,
}

impl SearchStats {
    fn init(&mut self) {
        self.nodes = 0;
        self.qnodes = 0;
        self.current_depth = 0;
        self.start_time = Instant::now();
    }

    fn nps(&self) -> i64 {
        let ms = self.start_time.elapsed().as_millis();
        if ms == 0 {
            return 0;
        }
        (self.nodes + self.qnodes) * 1000 / ms as i64
    }
}

static mut SEARCH_STATS: LazyLock<SearchStats> = LazyLock::new(|| SearchStats {
    nodes: 0,
    qnodes: 0,
    current_depth: 0,
    start_time: Instant::now(),
});

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
    fn init(&mut self) {
        for side in 0..2 {
            for piece in 0..6 {
                self.pieces[side][piece] = 0;
            }
        }
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
            if self.pieces[c][KING] != 0 {
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
        }

        let center = 0x0000001818000000;
        eval += (self.pieces[WHITE][PAWN] & center).count_ones() as i32 * 20;
        eval -= (self.pieces[BLACK][PAWN] & center).count_ones() as i32 * 20;

        let wpawns = self.pieces[WHITE][PAWN];
        for sq in wpawns.trailing_zeros() as usize..64 {
            let rank = sq / 8;
            if rank >= 4 {
                eval += (rank as i32 - 3) * 15;
            }
        }

        let bpawns = self.pieces[BLACK][PAWN];
        for sq in bpawns.trailing_zeros() as usize..64 {
            let rank = sq / 8;
            if rank <= 3 {
                eval -= (4 - rank as i32) * 15;
            }
        }

        if self.side == WHITE {
            eval
        } else {
            -eval
        }
    }
}

static mut ZOBRIST_PIECES: [[[U64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
static mut ZOBRIST_CASTLE: [U64; 16] = [0; 16];
static mut ZOBRIST_EP: [U64; 64] = [0; 64];
static mut ZOBRIST_SIDE: U64 = 0;

fn init_zobrist() {
    unsafe {
        ZOBRIST_SIDE = 0;
        for c in 0..2 {
            for p in 0..6 {
                for sq in 0..64 {
                    ZOBRIST_PIECES[c][p][sq] = 0;
                }
            }
        }
        for i in 0..16 {
            ZOBRIST_CASTLE[i] = 0;
        }
        for i in 0..64 {
            ZOBRIST_EP[i] = 0;
        }
    }
}

fn zobrist_hash(b: &Board) -> U64 {
    let mut hash = 0;

    for c in 0..2 {
        for p in 0..6 {
            let mut bb = b.pieces[c][p];
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                hash ^= unsafe { ZOBRIST_PIECES[c][p][sq] };
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

    for r in (tr + 1)..=7 {
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
    for f in (tf + 1)..=7 {
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

    for (r, f) in ((tr + 1)..=7).zip((tf + 1)..=7) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for (r, f) in ((tr + 1)..=7).zip((0..tf).rev()) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for (r, f) in ((0..tr).rev()).zip((tf + 1)..=7) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    for (r, f) in ((0..tr).rev()).zip((0..tf).rev()) {
        attacks |= 1 << (r * 8 + f);
        if (1 << (r * 8 + f)) & blockers != 0 {
            break;
        }
    }
    attacks
}

fn is_attacked(sq: i32, attacker: usize, b: &Board) -> bool {
    if attacker == WHITE {
        if sq >= 9 && sq % 8 != 0 && (1 << (sq - 9)) & b.pieces[WHITE][PAWN] != 0 {
            return true;
        }
        if sq >= 7 && sq % 8 != 7 && (1 << (sq - 7)) & b.pieces[WHITE][PAWN] != 0 {
            return true;
        }
    } else {
        if sq <= 56 && sq % 8 != 0 && (1 << (sq + 7)) & b.pieces[BLACK][PAWN] != 0 {
            return true;
        }
        if sq <= 54 && sq % 8 != 7 && (1 << (sq + 9)) & b.pieces[BLACK][PAWN] != 0 {
            return true;
        }
    }

    let bb = b.all;

    if unsafe { KING_MOVES[sq as usize] & b.pieces[attacker][KING] != 0 }
        || unsafe { KNIGHT_MOVES[sq as usize] & b.pieces[attacker][KNIGHT] != 0 }
    {
        return true;
    }

    if get_rook_attacks(sq, bb) & (b.pieces[attacker][ROOK] | b.pieces[attacker][QUEEN]) != 0 {
        return true;
    }
    if get_bishop_attacks(sq, bb)
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

    if m.piece == PAWN as i32 {
        if m.to == prev_ep as usize {
            let captured_pawn_sq = if b.side == WHITE {
                m.to - 8
            } else {
                m.to + 8
            };
            b.pieces[opponent][PAWN] ^= 1 << captured_pawn_sq;
            b.hash ^= unsafe { ZOBRIST_PIECES[opponent][PAWN][captured_pawn_sq as usize] };
        }
        if (m.from.wrapping_sub(m.to)) as i32 == 16 {
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
            b.hash ^= unsafe { ZOBRIST_PIECES[b.side][PAWN][m.to as usize] };
            b.hash ^= unsafe { ZOBRIST_PIECES[b.side][m.promo as usize][m.to as usize] };
        }
    } else if m.piece == KING as i32 {
        if (m.from.wrapping_sub(m.to)) as i32 == 2 {
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

fn is_legal_move(b: &Board, m: &Move) -> bool {
    let mut copy = b.clone();
    make_move(&mut copy, m);
    if copy.pieces[b.side][KING] == 0 {
        return false;
    }
    let king_sq = copy.pieces[b.side][KING].trailing_zeros() as i32;
    !is_attacked(king_sq, copy.side, &copy)
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

fn generate_moves(b: &Board, captures_only: bool) -> Vec<Move> {
    let mut moves = Vec::with_capacity(if captures_only { 32 } else { 128 });
    let mut bitboard;
    let mut attacks;

    for p in PAWN..=KING {
        bitboard = b.pieces[b.side][p];
        while bitboard != 0 {
            let from = bitboard.trailing_zeros() as usize;
            attacks = 0;

            if p == PAWN {
                let dir = if b.side == WHITE { 8 } else { -8 };
                let promo_rank = if b.side == WHITE { 7 } else { 0 };

                if !captures_only {
                    let to_sq = from as i32 + dir;
                    if to_sq >= 0 && to_sq < 64 && (b.all & (1 << to_sq)) == 0 {
                        if (to_sq / 8) == promo_rank {
                            moves.push(Move::new(from, to_sq as usize, PAWN as i32, -1, QUEEN as i32));
                        } else {
                            moves.push(Move::new(from, to_sq as usize, PAWN as i32, -1, 0));
                            let start_rank = if b.side == WHITE { 1 } else { 6 };
                            if (from / 8) == start_rank {
                                let to_sq2 = from as i32 + 2 * dir;
                                if (b.all & (1 << to_sq2)) == 0 {
                                    moves.push(Move::new(from, to_sq2 as usize, PAWN as i32, -1, 0));
                                }
                            }
                        }
                    }
                }

                let cap_dirs = [dir - 1, dir + 1];
                for &d in &cap_dirs {
                    let to = from as i32 + d;
                    if to < 0 || to > 63 || ((from % 8) as i32 - (to % 8)).abs() > 1 {
                        continue;
                    }

                    if (b.occupied[1 - b.side] & (1 << to)) != 0 {
                        if (to / 8) == promo_rank {
                            moves.push(Move::new(from, to as usize, PAWN as i32, 0, QUEEN as i32));
                        } else {
                            moves.push(Move::new(from, to as usize, PAWN as i32, 0, 0));
                        }
                    } else if !captures_only && to == b.ep {
                        moves.push(Move::new(from, to as usize, PAWN as i32, 0, 0));
                    }
                }
            } else if p == KING && !captures_only {
                if !is_in_check(b) {
                    if b.side == WHITE {
                        if (b.castle & 1) != 0 && (b.all & 0x60) == 0 {
                            if !is_attacked(5, BLACK, b) && !is_attacked(6, BLACK, b) {
                                moves.push(Move::new(4, 6, KING as i32, -1, 0));
                            }
                        }
                        if (b.castle & 2) != 0 && (b.all & 0xE) == 0 {
                            if !is_attacked(3, BLACK, b) && !is_attacked(2, BLACK, b) {
                                moves.push(Move::new(4, 2, KING as i32, -1, 0));
                            }
                        }
                    } else {
                        if (b.castle & 4) != 0 && (b.all & 0x6000000000000000) == 0 {
                            if !is_attacked(61, WHITE, b) && !is_attacked(62, WHITE, b) {
                                moves.push(Move::new(60, 62, KING as i32, -1, 0));
                            }
                        }
                        if (b.castle & 8) != 0 && (b.all & 0xE00000000000000) == 0 {
                            if !is_attacked(59, WHITE, b) && !is_attacked(58, WHITE, b) {
                                moves.push(Move::new(60, 58, KING as i32, -1, 0));
                            }
                        }
                    }
                }
            } else {
                unsafe {
                    if p == KNIGHT {
                        attacks = KNIGHT_MOVES[from];
                    } else if p == BISHOP {
                        attacks = get_bishop_attacks(from as i32, b.all);
                    } else if p == ROOK {
                        attacks = get_rook_attacks(from as i32, b.all);
                    } else if p == QUEEN {
                        attacks = get_rook_attacks(from as i32, b.all) | get_bishop_attacks(from as i32, b.all);
                    } else if p == KING {
                        attacks = KING_MOVES[from];
                    }

                    if captures_only {
                        attacks &= b.occupied[1 - b.side];
                    } else {
                        attacks &= !b.occupied[b.side];
                    }
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

    let mut legal_moves = Vec::with_capacity(moves.len());

    for m in moves.iter() {
        if is_legal_move(&b.clone(), m) {
            legal_moves.push(*m);
        }
    }

    legal_moves
}

fn score_moves(moves: &mut Vec<Move>, b: &Board, tt_move: Option<&Move>, ply: usize) {
    unsafe {
        for m in moves.iter_mut() {
            if let Some(tt_move) = tt_move {
                if *m == *tt_move {
                    m.score = 1_000_000;
                    continue;
                }
            }

            if b.occupied[1 - b.side] & (1 << m.to) != 0 {
                let victim_values = [100, 300, 300, 500, 900, 10000];
                for p in (PAWN..=KING).rev() {
                    if b.pieces[1 - b.side][p] & (1 << m.to) != 0 {
                        m.score = 100_000 + victim_values[p] * 10 - victim_values[m.piece as usize];
                        break;
                    }
                }
            } else if KILLER_MOVES.is_killer(m as *const Move, ply) {
                m.score = 90_000;
            } else {
                m.score = HISTORY_TABLE.get(b.side, m.from, m.to);
            }

            if m.promo == QUEEN as i32 {
                m.score += 80_000;
            }
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

    for m in captures.iter() {
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

fn search(
    b: &Board,
    mut depth: i32,
    mut alpha: i32,
    beta: i32,
    best_move: &mut Move,
    ply: usize,
    null_move: bool,
) -> i32 {
    unsafe {
        SEARCH_STATS.nodes += 1;
    }

    let in_check = is_in_check(b);
    if in_check {
        depth += 1;
    }

    let tt_index = (b.hash % TT_SIZE as u64) as usize;
    unsafe {
        let tt_entry = &TRANSPOSITION_TABLE[tt_index];
        let mut tt_move: Option<Move> = None;

        if tt_entry.hash == b.hash && tt_entry.depth >= depth {
            if tt_entry.flag == TT_EXACT {
                if ply == 0 {
                    *best_move = tt_entry.best_move.unwrap();
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

        if tt_entry.hash == b.hash && tt_entry.best_move.is_some() {
            tt_move = tt_entry.best_move;
        }

        if depth <= 0 {
            return quiescence(b, alpha, beta, 0);
        }

        if null_move && !in_check && depth >= 3 && ply > 0 {
            let mut copy = b.clone();
            copy.side = 1 - copy.side;
            copy.hash ^= ZOBRIST_SIDE;
            copy.ep = -1;

            let dummy = &mut Move::new(0, 0, 0, 0, 0);
            let r = if depth > 6 { 3 } else { 2 };
            let score = -search(&copy, depth - 1 - r, -beta, -beta + 1, dummy, ply + 1, false);

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

        score_moves(&mut moves, b, tt_move.as_ref(), ply);

        if ply == 0 && !moves.is_empty() {
            *best_move = moves[0];
        }

        let orig_alpha = alpha;

        for (move_count, m) in moves.iter().enumerate() {
            let mut reduction = 0i32;
            if move_count > 4
                && depth >= 3
                && !in_check
                && (b.occupied[1 - b.side] & (1 << m.to)) == 0
                && m.promo == 0
            {
                reduction = if move_count > 12 { 3 } else if move_count > 6 { 2 } else { 1 };

                if KILLER_MOVES.is_killer(m as *const Move, ply)
                    || HISTORY_TABLE.get(b.side, m.from, m.to) > 5000
                {
                    reduction = reduction.saturating_sub(1);
                }
            }

            let mut copy = b.clone();
            make_move(&mut copy, m);

            let mut score = if move_count == 0 {
                search(&copy, depth - 1 - reduction, -beta, -alpha, &mut Move::new(0, 0, 0, 0, 0), ply + 1, true)
            } else {
                let mut score = search(&copy, depth - 1 - reduction, -alpha - 1, -alpha, &mut Move::new(0, 0, 0, 0, 0), ply + 1, true);
                if score > alpha && score < beta {
                    score = search(&copy, depth - 1, -beta, -alpha, &mut Move::new(0, 0, 0, 0, 0), ply + 1, true);
                }
                score
            };

            if reduction > 0 && score > alpha {
                score = search(&copy, depth - 1, -beta, -alpha, &mut Move::new(0, 0, 0, 0, 0), ply + 1, true);
            }

            if score > alpha {
                alpha = score;
                if ply == 0 {
                    *best_move = *m;
                }

                if (b.occupied[1 - b.side] & (1 << m.to)) == 0 {
                    HISTORY_TABLE.update(b.side, m.from, m.to, depth);
                }
            }

            if alpha >= beta {
                if (b.occupied[1 - b.side] & (1 << m.to)) == 0 {
                    KILLER_MOVES.update(m as *const Move, ply);
                }
                break;
            }

            if depth <= 2
                && !in_check
                && move_count > 8
                && (b.occupied[1 - b.side] & (1 << m.to)) == 0
            {
                let futility_margin = depth * 100;
                if b.evaluate() + futility_margin < alpha {
                    break;
                }
            }
        }

        let tt_entry = &mut TRANSPOSITION_TABLE[tt_index];
        tt_entry.hash = b.hash;
        tt_entry.depth = depth;
        tt_entry.score = alpha;
        tt_entry.best_move = Some(*best_move);

        if alpha <= orig_alpha {
            tt_entry.flag = TT_ALPHA;
        } else if alpha >= beta {
            tt_entry.flag = TT_BETA;
        } else {
            tt_entry.flag = TT_EXACT;
        }

        alpha
    }
}

fn iterative_deepening(b: &Board, max_depth: usize, best_move: &mut Move, time_limit: Option<Duration>) -> i32 {
    let mut score = 0;
    let mut alpha = -INF;
    let mut beta = INF;
    let mut window = 50;

    unsafe {
        SEARCH_STATS.init();
    }

    for depth in 1..=max_depth {
        unsafe {
            SEARCH_STATS.current_depth = depth as i32;
        }

        if depth >= 4 {
            alpha = score - window;
            beta = score + window;
        }

        let temp_score = search(b, depth as i32, alpha, beta, best_move, 0, true);

        if temp_score <= alpha || temp_score >= beta {
            score = search(b, depth as i32, -INF, INF, best_move, 0, true);
            window = 50;
        } else {
            window = 25;
            score = temp_score;
        }

        if let Some(limit) = time_limit {
            unsafe {
                if SEARCH_STATS.start_time.elapsed() > limit * 4 / 10 && depth > 4 {
                    break;
                }
            }
        }

        println!("info depth {}", depth);
        print!(" score ");

        if score.abs() >= MATE - 1000 {
            let mate_in = (MATE - score.abs() + 1) / 2;
            print!("mate {}", if score < 0 { -mate_in } else { mate_in });
        } else {
            print!("cp {}", score);
        }

        unsafe {
            print!(" nodes {} nps {} pv ", SEARCH_STATS.nodes, SEARCH_STATS.nps());
        }

        let move_str = format!(
            "{}{}{}{}",
            (best_move.from % 8) as u8 + b'a',
            (best_move.from / 8) as u8 + b'1',
            (best_move.to % 8) as u8 + b'a',
            (best_move.to / 8) as u8 + b'1'
        );

        let move_str = if best_move.promo != 0 {
            let promo = match best_move.promo {
                4 => "q",
                3 => "r",
                2 => "b",
                1 => "n",
                _ => "",
            };
            format!("{}{}", move_str, promo)
        } else {
            move_str
        };

        println!("{}", move_str);

        if score.abs() >= MATE - 1000 {
            break;
        }
    }

    score
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
                        KING_MOVES[sq] |= 1 << (ny * 8 + nx);
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
                    KNIGHT_MOVES[sq] |= 1 << (ny * 8 + nx);
                }
            }
        }

        init_zobrist();
        HISTORY_TABLE.init();
        KILLER_MOVES.init();

        for tt in TRANSPOSITION_TABLE.iter_mut() {
            *tt = TTEntry::default();
        }
    }
}

fn parse_move(b: &Board, move_str: &str, parsed_move: &mut Move) -> bool {
    let moves = generate_moves(b, false);
    let from = ((move_str.as_bytes()[0] - b'a') + (move_str.as_bytes()[1] - b'1') * 8) as usize;
    let to = ((move_str.as_bytes()[2] - b'a') + (move_str.as_bytes()[3] - b'1') * 8) as usize;
    let promo_piece = if move_str.len() == 5 {
        match move_str.as_bytes()[4] as char {
            'q' => 4,
            'r' => 3,
            'b' => 2,
            'n' => 1,
            _ => 0,
        }
    } else {
        0
    };

    for m in moves {
        if m.from == from && m.to == to && (m.promo == 0 || m.promo == promo_piece) {
            *parsed_move = m;
            return true;
        }
    }

    false
}

fn main() {
    init_tables();
    let mut board = Board {
        pieces: [[0; 6]; 2],
        occupied: [0; 2],
        all: 0,
        side: WHITE,
        ep: -1,
        castle: 15,
        hash: 0,
    };
    board.init();

    let mut line = String::new();
    let mut cmd = String::new();

    while std::io::stdin().read_line(&mut line).unwrap() > 0 {
        if let Some(space_idx) = line.find(' ') {
            cmd = line[..space_idx].trim().to_string();
        } else {
            cmd = line.trim().to_string();
        }

        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        match &*cmd {
            "uci" => {
                println!("id name NanoChessTurbo");
                println!("id author CrvProject");
                println!("option name Depth type spin default 10 min 1 max 30");
                println!("option name Hash type spin default 64 min 1 max 1024");
                println!("uciok");
            }
            "setoption" => {
                if let Some(pos) = parts.iter().position(|&x| x == "name") {
                    let option_name: String = parts[pos + 1..]
                        .iter()
                        .take_while(|&&x| x != "value")
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ");
                    if let Some(value_pos) = parts.iter().position(|&x| x == "value") {
                        let value = parts[value_pos + 1].parse::<i32>().unwrap_or_default();
                        if option_name == "Depth" {
                            unsafe {
                                UCI_OPTIONS.depth = value.clamp(1, 30) as usize;
                            }
                        }
                    }
                }
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                board.init();
                unsafe {
                    HISTORY_TABLE.init();
                    KILLER_MOVES.init();
                }
                unsafe {
                    for tt in TRANSPOSITION_TABLE.iter_mut() {
                        *tt = TTEntry::default();
                    }
                }
            }
            "position" => {
                let sub_cmd = parts.get(1).unwrap_or(&"");
                if sub_cmd == &"startpos" {
                    board.init();
                } else if sub_cmd == &"fen" {
                    board.init();
                }
                if parts.contains(&"moves") {
                    if let Some(move_pos) = parts.iter().position(|&x| x == "moves") {
                        for m_str in parts.iter().skip(move_pos + 1) {
                            let mut m = Move::new(0, 0, 0, 0, 0);
                            if parse_move(&board, m_str, &mut m) {
                                make_move(&mut board, &m);
                            }
                        }
                    }
                }
            }
            "go" => {
                let mut search_depth = 0;
                let mut move_time = 0;
                let mut wtime = 0;
                let mut btime = 0;
                let mut winc = 0;
                let mut binc = 0;
                let mut movestogo = 40;
                let mut infinite = false;

                for (i, &part) in parts.iter().enumerate() {
                    match part {
                        "depth" => {
                            if let Some(next) = parts.get(i + 1) {
                                search_depth = next.parse().unwrap_or_default();
                                search_depth = search_depth.clamp(1, 30);
                            }
                        }
                        "movetime" => {
                            if let Some(next) = parts.get(i + 1) {
                                move_time = next.parse().unwrap_or_default();
                            }
                        }
                        "wtime" => {
                            if let Some(next) = parts.get(i + 1) {
                                wtime = next.parse().unwrap_or_default();
                            }
                        }
                        "btime" => {
                            if let Some(next) = parts.get(i + 1) {
                                btime = next.parse().unwrap_or_default();
                            }
                        }
                        "winc" => {
                            if let Some(next) = parts.get(i + 1) {
                                winc = next.parse().unwrap_or_default();
                            }
                        }
                        "binc" => {
                            if let Some(next) = parts.get(i + 1) {
                                binc = next.parse().unwrap_or_default();
                            }
                        }
                        "movestogo" => {
                            if let Some(next) = parts.get(i + 1) {
                                movestogo = next.parse().unwrap_or_default();
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

                    allocated_time = time_left / movestogo + (increment as f64 * 0.8) as i32;
                    allocated_time = allocated_time.min(time_left / 3);
                } else if move_time > 0 {
                    allocated_time = (move_time as f64 * 0.95) as i32;
                }

                let mut best_move = Move::new(0, 0, 0, 0, 0);
                let time_limit = if allocated_time > 0 {
                    Some(Duration::from_millis(allocated_time as u64))
                } else {
                    None
                };
                iterative_deepening(&board, search_depth, &mut best_move, time_limit);

                if best_move.from != best_move.to || best_move.from != 0 {
                    let move_str = format!(
                        "{}{}{}{}",
                        (best_move.from % 8) as u8 + b'a',
                        (best_move.from / 8) as u8 + b'1',
                        (best_move.to % 8) as u8 + b'a',
                        (best_move.to / 8) as u8 + b'1'
                    );

                    let move_str = if best_move.promo != 0 {
                        let promo = match best_move.promo {
                            4 => "q",
                            3 => "r",
                            2 => "b",
                            1 => "n",
                            _ => "",
                        };
                        format!("{}{}", move_str, promo)
                    } else {
                        move_str
                    };

                    println!("bestmove {}", move_str);
                } else {
                    let moves = generate_moves(&board, false);
                    if let Some(fallback) = moves.get(0) {
                        let move_str = format!(
                            "{}{}{}{}",
                            (fallback.from % 8) as u8 + b'a',
                            (fallback.from / 8) as u8 + b'1',
                            (fallback.to % 8) as u8 + b'a',
                            (fallback.to / 8) as u8 + b'1'
                        );

                        let move_str = if fallback.promo != 0 {
                            let promo = match fallback.promo {
                                4 => "q",
                                3 => "r",
                                2 => "b",
                                1 => "n",
                                _ => "",
                            };
                            format!("{}{}", move_str, promo)
                        } else {
                            move_str
                        };

                        println!("bestmove {}", move_str);
                    } else {
                        println!("bestmove 0000");
                    }
                }
            }
            "quit" => {
                break;
            }
            _ => {}
        }
        line.clear();
    }
}