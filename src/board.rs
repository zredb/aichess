use toto::Toi32;

use crate::MAX_CELLS_SIZE;

const RANK_TOP: usize = 3;
const RANK_BOTTOM: usize = 12;
const FILE_LEFT: usize = 3;
const FILE_CENTER: usize = 7;
const FILE_RIGHT: usize = 11;
const cszPieceBytes: &str = "KABNRCP";

fn piece_byte(pt: usize) -> char {
    cszPieceBytes.chars().nth(pt).unwrap()
}


const CBC_IN_BOARD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

const CBC_IN_FORT: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

const CBC_CAN_PROMOTE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

const CC_LEGAL_SPAN_TAB: [u8; 512] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0
];

const CC_KNIGHT_PIN_TAB: [i8; 512] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, -16, 0, -16, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, -1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, -1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 16, 0, 16, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0
];

/* 棋子序号对应的棋子类型
 *
 * ElephantEye的棋子序号从0到47，其中0到15不用，16到31表示红子，32到47表示黑子。
 * 每方的棋子顺序依次是：帅仕仕相相马马车车炮炮兵兵兵兵兵(将士士象象马马车车炮炮卒卒卒卒卒)
 * 提示：判断棋子是红子用"pc < 32"，黑子用"pc >= 32"
 */
const cnPieceTypes: [u8; 48] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 6,
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 6
];

fn piece_type(pc: usize) -> u8 {
    cnPieceTypes[pc]
}

fn in_board(sq: usize) -> bool {
    CBC_IN_BOARD[sq] == 1
}

fn in_fort(sq: usize) -> bool {
    CBC_IN_FORT[sq] == 1
}

fn can_promote(sq: usize) -> bool {
    CBC_CAN_PROMOTE[sq] == 1
}

fn legal_span_tab(n_disp: usize) -> u8 {
    CC_LEGAL_SPAN_TAB[n_disp]
}

fn knight_pin_tab(n_disp: usize) -> i8 {
    CC_KNIGHT_PIN_TAB[n_disp]
}

fn rank_y(sq: usize) -> usize {
    sq >> 4
}

fn file_x(sq: usize) -> usize {
    sq & 15
}

fn coord_xy(x: usize, y: usize) -> usize {
    x + (y << 4)
}

fn square_flip(sq: usize) -> usize {
    254 - sq
}

fn file_flip(x: usize) -> usize {
    14 - x
}

fn rank_flip(y: usize) -> usize {
    15 - y
}

fn opp_side(sd: usize) -> usize {
    1 - sd
}

fn square_forward(sq: usize, sd: usize) -> usize {
    sq - 16 + (sd << 5)
}

fn square_backward(sq: usize, sd: usize) -> usize {
    sq + 16 - (sd << 5)
}

fn king_span(sq_src: usize, sq_dst: usize) -> bool {
    legal_span_tab(sq_dst - sq_src + 256) == 1
}

fn advisor_span(sq_src: usize, sq_dst: usize) -> bool {
    legal_span_tab(sq_dst - sq_src + 256) == 2
}

fn bishop_span(sq_src: usize, sq_dst: usize) -> bool {
    legal_span_tab(sq_dst - sq_src + 256) == 3
}

fn bishop_pin(sq_src: usize, sq_dst: usize) -> usize {
    (sq_src + sq_dst) >> 1
}

fn knight_pin(sq_src: usize, sq_dst: usize) -> u8 {
    ((sq_src as u32).to_i32() + CC_KNIGHT_PIN_TAB[sq_dst - sq_src + 256].to_i32()) as u8
}

fn white_half(sq: usize) -> bool {
    (sq & 0x80) != 0
}

fn black_half(sq: usize) -> bool {
    (sq & 0x80) == 0
}

fn home_half(sq: usize, sd: usize) -> bool {
    (sq & 0x80) != (sd << 7)
}

fn away_half(sq: usize, sd: usize) -> bool {
    (sq & 0x80) == (sd << 7)
}

fn same_half(sq_src: usize, sq_dst: usize) -> bool {
    ((sq_src ^ sq_dst) & 0x80) == 0
}

fn diff_half(sq_src: usize, sq_dst: usize) -> bool {
    ((sq_src ^ sq_dst) & 0x80) != 0
}

fn rank_disp(y: usize) -> usize {
    y << 4
}

fn file_disp(x: usize) -> usize {
    x
}

const KING_TYPE: usize = 0;
const ADVISOR_TYPE: usize = 1;
const BISHOP_TYPE: usize = 2;
const KNIGHT_TYPE: usize = 3;
const ROOK_TYPE: usize = 4;
const CANNON_TYPE: usize = 5;
const PAWN_TYPE: usize = 6;

const KING_FROM: usize = 0;
const ADVISOR_FROM: usize = 1;
const ADVISOR_TO: usize = 2;
const BISHOP_FROM: usize = 3;
const BISHOP_TO: usize = 4;
const KNIGHT_FROM: usize = 5;
const KNIGHT_TO: usize = 6;
const ROOK_FROM: usize = 7;
const ROOK_TO: usize = 8;
const CANNON_FROM: usize = 9;
const CANNON_TO: usize = 10;
const PAWN_FROM: usize = 11;
const PAWN_TO: usize = 15;

const REP_NONE: usize = 0;
const REP_DRAW: usize = 1;
const REP_LOSS: usize = 3;
const REP_WIN: usize = 5;

const KING_BITPIECE: u32 = 1 << KING_FROM;
const ADVISOR_BITPIECE: u32 = (1 << ADVISOR_FROM) | (1 << ADVISOR_TO);
const BISHOP_BITPIECE: u32 = (1 << BISHOP_FROM) | (1 << BISHOP_TO);
const KNIGHT_BITPIECE: u32 = (1 << KNIGHT_FROM) | (1 << KNIGHT_TO);
const ROOK_BITPIECE: u32 = (1 << ROOK_FROM) | (1 << ROOK_TO);
const CANNON_BITPIECE: u32 = (1 << CANNON_FROM) | (1 << CANNON_TO);
const PAWN_BITPIECE: u32 = (1 << PAWN_FROM) | (1 << (PAWN_FROM + 1)) |
    (1 << (PAWN_FROM + 2)) | (1 << (PAWN_FROM + 3)) | (1 << PAWN_TO);
const ATTACK_BITPIECE: u32 = KNIGHT_BITPIECE | ROOK_BITPIECE | CANNON_BITPIECE | PAWN_BITPIECE;

fn bit_piece(pc: usize) -> u32 {
    1 << (pc - 16)
}

fn white_bitpiece(n_bitpiece: u32) -> u32 {
    n_bitpiece
}

fn black_bitpiece(n_bitpiece: u32) -> u32 {
    n_bitpiece << 16
}

fn both_bitpiece(n_bitpiece: u32) -> u32 {
    n_bitpiece + (n_bitpiece << 16)
}


fn side_tag(sd: usize) -> usize {
    16 + (sd << 4)
}

fn opp_side_tag(sd: usize) -> usize {
    32 - (sd << 4)
}


fn piece_index(pc: usize) -> usize {
    pc & 15
}

fn src(mv: usize) -> usize {
    mv & 255
}

fn dst(mv: usize) -> usize {
    mv >> 8
}

fn mv(sq_src: usize, sq_dst: usize) -> usize {
    sq_src + (sq_dst << 8)
}

// fn move_coord(mv: usize) -> u32 {
//     let mut ret = [0u8; 4];
//     ret[0] = (file_x(src(mv)) - FILE_LEFT + b'a') as u8;
//     ret[1] = (b'9' - rank_y(src(mv)) + RANK_TOP) as u8;
//     ret[2] = (file_x(dst(mv)) - FILE_LEFT + b'a') as u8;
//     ret[3] = (b'9' - rank_y(dst(mv)) + RANK_TOP) as u8;
//     // 断言输出着法的合理性
//     assert!(ret[0] >= b'a' && ret[0] <= b'i');
//     assert!(ret[1] >= b'0' && ret[1] <= b'9');
//     assert!(ret[2] >= b'a' && ret[2] <= b'i');
//     assert!(ret[3] >= b'0' && ret[3] <= b'9');
//     u32::from_le_bytes(ret)
// }
//
// fn coord_move(dw_move_str: u32) -> usize {
//     let lp_arg_ptr = dw_move_str.to_le_bytes();
//     let sq_src = coord_xy((lp_arg_ptr[0] - b'a' + FILE_LEFT) as usize, (b'9' - lp_arg_ptr[1] + RANK_TOP) as usize);
//     let sq_dst = coord_xy((lp_arg_ptr[2] - b'a' + FILE_LEFT) as usize, (b'9' - lp_arg_ptr[3] + RANK_TOP) as usize);
//     // 对输入着法的合理性不作断言
//     // assert!(in_board(sq_src) && in_board(sq_dst));
//     if in_board(sq_src) && in_board(sq_dst) {
//         mv(sq_src, sq_dst)
//     } else {
//         0
//     }
// }

///局面
pub struct Position {
    // 轮到哪方走，0表示红方，1表示黑方
    sdPlayer: i32,
    // 每个格子放的棋子，None表示没有棋子
    ucpc_squares: [u8; 256],
    // 每个棋子放的位置，0表示被吃
    ucsq_pieces: [u8; 48],
}

impl Position {
    /// 新建一个棋盘对象。
    pub fn new() -> Position {
        Position {
            sdPlayer: 0,
            ucpc_squares: [None; MAX_CELLS_SIZE],
            ucsq_pieces: [0; 48],
        }
    }

    // pub(crate) fn search_piece_locations(&self, piece: char) -> Vec<usize> {
    //     let mut res = vec![];
    //     for (idx, pos) in self.positions.iter().enumerate() {
    //         if let Some(p) = pos {
    //             if p == &piece {
    //                 res.push(idx);
    //             }
    //         }
    //     }
    //     res
    // }

    fn from_fen(sz_fen: &str) -> Self {
        let mut pc_white = [0; 7];
        let mut pc_black = [0; 7];
        let mut lp_fen = sz_fen.chars().peekable();

        pc_white[0] = side_tag(0) + KING_FROM;
        pc_white[1] = side_tag(0) + ADVISOR_FROM;
        pc_white[2] = side_tag(0) + BISHOP_FROM;
        pc_white[3] = side_tag(0) + KNIGHT_FROM;
        pc_white[4] = side_tag(0) + ROOK_FROM;
        pc_white[5] = side_tag(0) + CANNON_FROM;
        pc_white[6] = side_tag(0) + PAWN_FROM;
        for i in 0..7 {
            pc_black[i] = pc_white[i] + 16;
        }
        let mut position = Position::new();

        if lp_fen.peek().is_none() {
            return position;
        }

        let mut i = RANK_TOP;
        let mut j = FILE_LEFT;

        while let Some(ch) = lp_fen.next() {
            if ch == '/' {
                j = FILE_LEFT;
                i += 1;
                if i > RANK_BOTTOM {
                    break;
                }
            } else if ch.is_digit(10) {
                j += ch.to_digit(10).unwrap() as usize;
            } else if ch.is_ascii_uppercase() {
                if j <= FILE_RIGHT {
                    let k = fen_piece(ch) as usize;
                    if k < 7 && pc_white[k] < 32 {
                        if position.ucsq_pieces[pc_white[k]] == 0 {
                            position.add_piece(coord_xy(j, i), pc_white[k]);
                            pc_white[k] += 1;
                        }
                    }
                    j += 1;
                }
            } else if ch.is_ascii_lowercase() {
                if j <= FILE_RIGHT {
                    let k = fen_piece((ch as u8 + b'A' - b'a') as char) as usize;
                    if k < 7 {
                        if pc_black[k] < 48 {
                            if position.ucsq_pieces[pc_black[k]] == 0 {
                                position.add_piece(coord_xy(j, i), pc_black[k]);
                                pc_black[k] += 1;
                            }
                        }
                    }
                    j += 1;
                }
            }

            if lp_fen.peek().is_none() {
                return position;
            }
        }

        if let Some(ch) = lp_fen.next() {
            if ch == 'b' {
                position.change_side();
            }
        }

        position
    }
    fn change_side(&mut self){
        if self.sdPlayer==0 {
            self.sdPlayer=1;
        }else{
            self.sdPlayer=0;
        }
    }
    fn add_piece(&mut self, sq: usize, pc: usize) {
        self.ucpc_squares[sq] = pc as u8;
        self.ucsq_pieces[pc] = sq as u8;
    }

    fn del_piece(&mut self, sq: usize, pc: usize) {
        self.ucpc_squares[sq] = 0;
        self.ucsq_pieces[pc] = 0;
    }

    fn to_fen(position: &Position) -> String {
        let mut fen = String::new();

        for i in RANK_TOP..=RANK_BOTTOM {
            let mut k = 0;
            for j in FILE_LEFT..=FILE_RIGHT {
                let pc = position.ucpc_squares[coord_xy(j, i)];
                if pc != 0 {
                    if k > 0 {
                        fen.push((k + '0' as u8) as char);
                        k = 0;
                    }
                    fen.push(piece_byte(piece_type(pc)) + if pc < 32 { 0 } else { 'a' as u8 - 'A' as u8 } as u8 as char);
                } else {
                    k += 1;
                }
            }
            if k > 0 {
                fen.push((k + '0' as u8) as char);
            }
            fen.push('/');
        }

        fen.pop();
        fen.push(' ');

        if position.sd_player == 0 {
            fen.push('w');
        } else {
            fen.push('b');
        }

        fen
    }
}

fn fen_piece(n_arg: char) -> u32 {
    match n_arg {
        'K' => 1,
        'Q' | 'A' => 2,
        'B' | 'E' => 3,
        'N' | 'H' => 4,
        'R' => 5,
        'C' => 6,
        'P' => 7,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Position;

    #[test]
    fn test_gen_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";
        let board = Position::from_fen(fen);
        //let moves = board.generate_all_moves(&Side::Red);
    }
}