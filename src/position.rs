use std::collections::HashSet;
use toto::Toi32;

use crate::game::Side;
use crate::moves::Move;
use crate::pregen::{PreGen, SlideMask, SlideMove, Zobrist};

pub(crate) const RANK_TOP: usize = 3;
pub(crate) const RANK_BOTTOM: usize = 12;
pub(crate) const FILE_LEFT: usize = 3;
pub(crate) const FILE_CENTER: usize = 7;
pub(crate) const FILE_RIGHT: usize = 11;

const CBC_IN_BOARD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const CBC_IN_FORT: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const CBC_CAN_PROMOTE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const CC_LEGAL_SPAN_TAB: [u8; 512] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0,
    0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0,
    0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const CC_KNIGHT_PIN_TAB: [i8; 512] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    -16, 0, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 16, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
];

/* 棋子序号对应的棋子类型
 *
 * 棋子序号从0到47，其中0到15不用，16到31表示红子，32到47表示黑子。
 * 每方的棋子顺序依次是：帅仕仕相相马马车车炮炮兵兵兵兵兵(将士士象象马马车车炮炮卒卒卒卒卒)
 * 提示：判断棋子是红子用"pc < 32"，黑子用"pc >= 32"
 * 为什么不是0--32?
 */
const cnPieceTypes: [u8; 48] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 6,
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 6,
];

const cszPieceBytes: &str = "KABNRCP";

fn piece_char(pt: usize) -> char {
    cszPieceBytes.chars().nth(pt).unwrap()
}

fn piece_char_with_side(pc: usize) -> char {
    let pt = piece_type(pc);
    let mut pcc = piece_char(pt as usize);
    if pc >= 32usize {
        //黑方棋子
        pcc = pcc.to_ascii_lowercase();
    }
    pcc
}

fn piece_type(pc: usize) -> u8 {
    cnPieceTypes[pc]
}

pub(crate) fn in_board(sq: i32) -> bool {
    if !(0..=256).contains(&sq) {
        return false;
    }
    CBC_IN_BOARD[sq as usize] == 1
}

pub(crate) fn in_fort(sq: i32) -> bool {
    if !(0..=256).contains(&sq) {
        return false;
    }
    CBC_IN_FORT[sq as usize] == 1
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

pub fn rank_y(sq: usize) -> usize {
    sq >> 4
}

pub fn file_x(sq: usize) -> usize {
    sq & 15
}

pub(crate) fn coord_xy(x: usize, y: usize) -> usize {
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

// 格子水平镜像
pub(crate) fn square_forward(sq: i32, sd: i32) -> i32 {
    sq - 16 + (sd << 5)
}

// 格子水平镜像(反向)
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

pub(crate) fn bishop_pin(sq_src: i32, sq_dst: i32) -> u8 {
    ((sq_src + sq_dst) >> 1) as u8
}

pub(crate) fn knight_pin(sq_src: i32, sq_dst: i32) -> u8 {
    let c = (sq_dst - sq_src + 256) as usize;
    (sq_src + CC_KNIGHT_PIN_TAB[c].to_i32()) as u8
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

pub(crate) fn away_half(sq: i32, sd: i32) -> bool {
    (sq & 0x80) == (sd << 7)
}

pub(crate) fn same_half(sq_src: i32, sq_dst: i32) -> bool {
    ((sq_src ^ sq_dst) & 0x80) == 0
}

fn diff_half(sq_src: usize, sq_dst: usize) -> bool {
    ((sq_src ^ sq_dst) & 0x80) != 0
}

pub(crate) fn rank_disp(y: usize) -> u8 {
    (y << 4) as u8
}

pub(crate) fn file_disp(x: usize) -> u8 {
    x as u8
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
const PAWN_BITPIECE: u32 = (1 << PAWN_FROM)
    | (1 << (PAWN_FROM + 1))
    | (1 << (PAWN_FROM + 2))
    | (1 << (PAWN_FROM + 3))
    | (1 << PAWN_TO);
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

fn side_tag2(sd: &Side) -> u8 {
    match sd {
        Side::Red => 16,
        Side::Black => 32,
    }
}

fn opp_side_tag(sd: usize) -> usize {
    32 - (sd << 4)
}

fn opp_side_tag2(sd: &Side) -> usize {
    match sd {
        Side::Red => 32,
        Side::Black => 16,
    }
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

///局面
pub struct Position {
    // 轮到哪方走，0表示红方，1表示黑方
    sd_player: u32,
    current_player: Side,
    // 每个格子放的棋子，None表示没有棋子
    ucpc_squares: [u8; 256],
    // 每个棋子放的位置，0表示被吃
    ucsq_pieces: [u8; 48],
    dwBitPiece: u32,
    // 32位的棋子位，0到31位依次表示序号为16到47的棋子是否还在棋盘上
    wBitRanks: [u16; 16],
    // 位行数组，注意用法是"wBitRanks[RANK_Y(sq)]"
    wBitFiles: [u16; 16],
    // 位列数组，注意用法是"wBitFiles[FILE_X(sq)]"
    pre_gen: PreGen,
    zobr: Zobrist, // Zobrist
}

impl Position {
    /// 新建一个棋盘对象。
    pub fn new() -> Position {
        Position {
            sd_player: 0,
            current_player: Side::Red,
            ucpc_squares: [0; 256],
            ucsq_pieces: [0; 48],
            dwBitPiece: 0,
            wBitRanks: [0; 16],
            wBitFiles: [0; 16],
            pre_gen: PreGen::new(),
            zobr: Zobrist::init_rc4(),
        }
    }
    pub fn piece_loc(&self) -> Vec<(char, u8)> {
        let mut res = Vec::new();
        for (idx, pos) in self.ucsq_pieces.iter().enumerate().skip(16) {
            let pcc = piece_char_with_side(idx);
            res.push((pcc, *pos));
        }
        res
    }

    // pub(crate) fn search_piece_locations(&self, piece: char) -> Vec<usize> {
    //     let mut res = vec![];
    //     for (idx, pos) in pos.positions.iter().enumerate() {
    //         if let Some(p) = pos {
    //             if p == &piece {
    //                 res.push(idx);
    //             }
    //         }
    //     }
    //     res
    // }

    pub fn from_fen(sz_fen: &str) -> Self {
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
            } else if ch.is_ascii_digit() {
                j += ch.to_digit(10).unwrap() as usize;
            } else if ch.is_ascii_uppercase() {
                if j <= FILE_RIGHT {
                    let k = fen_piece(ch);
                    let pc = pc_white[k];
                    if k < 7 && pc < 32 && position.ucsq_pieces[pc] == 0 {
                        position.add_piece(coord_xy(j, i), pc);
                        pc_white[k] += 1;
                    }
                    j += 1;
                }
            } else if ch.is_ascii_lowercase() && j <= FILE_RIGHT {
                let k = fen_piece(ch.to_ascii_uppercase());
                let pc = pc_black[k];
                if k < 7 && pc < 48 && position.ucsq_pieces[pc] == 0 {
                    position.add_piece(coord_xy(j, i), pc);
                    pc_black[k] += 1;
                }
                j += 1;
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
    fn change_side(&mut self) {
        if self.sd_player == 0 {
            self.sd_player = 1;
        } else {
            self.sd_player = 0;
        }
    }
    fn change_side2(&mut self) {
        match self.current_player {
            Side::Red => self.current_player = Side::Black,
            Side::Black => self.current_player = Side::Black,
        }
    }
    fn add_piece(&mut self, sq: usize, pc: usize) {
        self.ucpc_squares[sq] = pc as u8;
        self.ucsq_pieces[pc] = sq as u8;
        self.wBitFiles[file_x(sq)] ^= self.pre_gen.w_bit_file_mask[sq];
        self.wBitRanks[rank_y(sq)] ^= self.pre_gen.w_bit_rank_mask[sq];
        self.dwBitPiece ^= bit_piece(pc);
        let mut ppt = piece_type(pc);
        if pc >= 32 {
            ppt += 7;
        }
        self.zobr.xor(&self.pre_gen.zobr_table[sq][ppt as usize]);
    }

    fn del_piece(&mut self, sq: usize, pc: usize) {
        self.ucpc_squares[sq] = 0;
        self.ucsq_pieces[pc] = 0;
        self.wBitFiles[file_x(sq)] ^= self.pre_gen.w_bit_file_mask[sq];
        self.wBitRanks[rank_y(sq)] ^= self.pre_gen.w_bit_rank_mask[sq];
        self.dwBitPiece ^= bit_piece(pc);
        let mut ppt = piece_type(pc);
        if pc >= 32 {
            ppt += 7;
        }
        self.zobr.xor(&self.pre_gen.zobr_table[ppt as usize][sq]);
    }

    fn to_fen(position: &Position) -> String {
        let mut fen = String::new();
        for i in RANK_TOP..=RANK_BOTTOM {
            let mut k = 0;
            for j in FILE_LEFT..=FILE_RIGHT {
                let pc = position.ucpc_squares[coord_xy(j, i)];
                if pc != 0 {
                    if k > 0 {
                        let x = char::from_digit(k, 10).unwrap();
                        fen.push(x);
                        k = 0;
                    }
                    let mut c = piece_char(piece_type(pc as usize) as usize);
                    if pc >= 32 {
                        //32到47表示黑子, Fen中用小写字母表示
                        c = c.to_ascii_lowercase();
                    }
                    fen.push(c);
                } else {
                    k += 1;
                }
            }
            if k > 0 {
                let x = char::from_digit(k, 10).unwrap();
                fen.push(x);
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
    fn wbit_piece(&self) -> u32 {
        match self.current_player {
            Side::Red => self.dwBitPiece & 0xffff,
            Side::Black => self.dwBitPiece >> 16,
        }
    }

    fn get_piece_char(&self, pt: usize) -> char {
        match self.current_player {
            Side::Red => piece_char(pt),
            Side::Black => piece_char(pt).to_ascii_lowercase(),
        }
    }

    fn rank_move(&self, x: usize, y: usize) -> &SlideMove {
        let adjusted_x = x - FILE_LEFT;
        let rank_move_tab = &self.pre_gen.smv_rank_move_tab[adjusted_x];
        let w_bit_ranks = &self.wBitRanks[y];
        let res = &rank_move_tab[*w_bit_ranks as usize];
        res
    }

    fn file_move(&self, x: usize, y: usize) -> &SlideMove {
        let adjusted_y = y - RANK_TOP;
        let file_move_tab = &self.pre_gen.smv_file_move_tab[adjusted_y];
        let w_bit_files = &self.wBitFiles[x];
        let res = &file_move_tab[*w_bit_files as usize];
        res
    }

    fn rank_mask(&self, x: usize, y: usize) -> &SlideMask {
        let adjusted_x = x - FILE_LEFT;
        let rank_move_tab = &self.pre_gen.sms_rank_mask_tab[adjusted_x];
        let w_bit_ranks = &self.wBitRanks[y];
        let res = &rank_move_tab[*w_bit_ranks as usize];
        res
    }

    fn file_mask(&self, x: usize, y: usize) -> &SlideMask {
        let adjusted_y = y - RANK_TOP;
        let file_mask_tab = &self.pre_gen.sms_file_mask_tab[adjusted_y];
        let w_bit_files = &self.wBitFiles[x];
        let res = &file_mask_tab[*w_bit_files as usize];
        res
    }

    fn gen_legal_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        res.append(&mut self.gen_cap_moves());
        res.append(&mut self.gen_nocap_moves());
        res
    }
    fn gen_king_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        let n_side_tag = side_tag2(&self.current_player);
        let n_opp_side_tag = opp_side_tag2(&self.current_player);
        // 1. 生成帅(将)的着法
        let sq_src = self.ucsq_pieces[n_side_tag as usize + KING_FROM];
        if sq_src != 0 {
            let lpucsq_dst = self.pre_gen.ucsq_king_moves[sq_src as usize];
            for sq_dst in lpucsq_dst {
                if sq_dst != 0 {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured == 0 {
                        //不吃子着法
                        res.push(Move::new(
                            self.get_piece_char(KING_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                    if (pc_captured & n_opp_side_tag as u8) != 0 {
                        // 吃子着法
                        res.push(Move::new(
                            self.get_piece_char(KING_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
            }
        }
        res
    }
    fn gen_advisor_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        todo!();
        res
    }
    fn gen_bishop_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        todo!();
        res
    }
    fn gen_knight_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        todo!();
        res
    }
    fn gen_rook_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        todo!();
        res
    }
    fn gen_canoon_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        todo!();
        res
    }
    fn gen_pawn_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        todo!();
        res
    }

    fn gen_cap_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        let n_side_tag = side_tag2(&self.current_player);
        let n_opp_side_tag = opp_side_tag2(&self.current_player);

        // 1. 生成帅(将)的着法
        let sq_src = self.ucsq_pieces[n_side_tag as usize + KING_FROM];
        if sq_src != 0 {
            let lpucsq_dst = self.pre_gen.ucsq_king_moves[sq_src as usize];
            for sq_dst in lpucsq_dst {
                if sq_dst != 0 {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if (pc_captured & n_opp_side_tag as u8) != 0 {
                        res.push(Move::new(
                            self.get_piece_char(KING_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                } else {
                    break;
                }
            }
        }

        // 2. 生成仕(士)的着法
        for i in ADVISOR_FROM..=ADVISOR_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst = &self.pre_gen.ucsq_advisor_moves[sq_src as usize];
                let mut sq_dst = *lpucsq_dst;
                for sq_dst in lpucsq_dst {
                    if sq_dst != &0 {
                        let pc_captured = self.ucpc_squares[*sq_dst as usize];
                        if (pc_captured & n_opp_side_tag as u8) != 0 {
                            res.push(Move::new(
                                self.get_piece_char(ADVISOR_TYPE),
                                sq_src as usize,
                                *sq_dst,
                            ));
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        // 3. 生成相(象)的着法
        for i in BISHOP_FROM..=BISHOP_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst = &self.pre_gen.ucsq_bishop_moves[sq_src as usize];
                let lpucsq_pin = &self.pre_gen.ucsq_bishop_pins[sq_src as usize];
                let mut sq_dst = *lpucsq_dst;
                for (sq_dst, pin) in lpucsq_dst.into_iter().zip(lpucsq_pin) {
                    if sq_dst != &0 {
                        if self.ucpc_squares[*pin as usize] == 0 {
                            let pc_captured = self.ucpc_squares[*sq_dst as usize];
                            if (pc_captured & n_opp_side_tag as u8) != 0 {
                                res.push(Move::new(
                                    self.get_piece_char(BISHOP_TYPE),
                                    sq_src as usize,
                                    *sq_dst,
                                ));
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        // 4. 生成马的着法
        for i in KNIGHT_FROM..=KNIGHT_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst = &self.pre_gen.ucsq_knight_moves[sq_src as usize];
                let lpucsq_pin = &self.pre_gen.ucsq_knight_pins[sq_src as usize];
                for (sq_dst, pin) in lpucsq_dst.into_iter().zip(lpucsq_pin) {
                    if self.ucpc_squares[*pin as usize] == 0 {
                        let pc_captured = self.ucpc_squares[*sq_dst as usize];
                        if (pc_captured & n_opp_side_tag as u8) != 0 {
                            res.push(Move::new(
                                self.get_piece_char(KNIGHT_TYPE),
                                sq_src as usize,
                                *sq_dst,
                            ));
                        }
                    }
                }
            }
        }
        // 5. 生成车的着法
        for i in ROOK_FROM..=ROOK_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let x = file_x(sq_src as usize);
                let y = rank_y(sq_src as usize);

                let lpsmv = self.rank_move(x, y);
                let sq_dst = lpsmv.ucRookCap[0] + rank_disp(y);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(ROOK_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
                let sq_dst = lpsmv.ucRookCap[1] + rank_disp(y);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(ROOK_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }

                let lpsmv = self.file_move(x, y);
                let sq_dst = lpsmv.ucRookCap[0] + file_disp(x);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(ROOK_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
                let sq_dst = lpsmv.ucRookCap[1] + file_disp(x);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(ROOK_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
            }
        }

        // 6. 生成炮的着法
        for i in CANNON_FROM..=CANNON_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let x = file_x(sq_src as usize);
                let y = rank_y(sq_src as usize);

                let lpsmv = self.rank_move(x, y);
                let sq_dst = lpsmv.ucCannonCap[0] + rank_disp(y);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(CANNON_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
                let sq_dst = lpsmv.ucCannonCap[1] + rank_disp(y);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(CANNON_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }

                let lpsmv = self.file_move(x, y);
                let sq_dst = lpsmv.ucCannonCap[0] + file_disp(x);
                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(CANNON_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
                let sq_dst = lpsmv.ucCannonCap[1] + file_disp(x);

                if sq_dst != sq_src {
                    let pc_captured = self.ucpc_squares[sq_dst as usize];
                    if pc_captured & n_opp_side_tag as u8 != 0 {
                        res.push(Move::new(
                            self.get_piece_char(CANNON_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
            }
        }

        // 7. 生成兵(卒)的着法
        for i in PAWN_FROM..=PAWN_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst =
                    self.pre_gen.ucsq_pawn_moves[self.sd_player as usize][sq_src as usize];
                for sq_dst in lpucsq_dst {
                    if sq_dst != 0 {
                        let pc_captured = self.ucpc_squares[sq_dst as usize];
                        if pc_captured & n_opp_side_tag as u8 != 0 {
                            res.push(Move::new(
                                self.get_piece_char(PAWN_TYPE),
                                sq_src as usize,
                                sq_dst,
                            ));
                        }
                    }
                }
            }
        }
        res
    }

    fn gen_nocap_moves(&self) -> Vec<Move> {
        let mut res = Vec::new();
        let n_side_tag = side_tag2(&self.current_player);
        // 1. 生成帅(将)的着法
        let sq_src = self.ucsq_pieces[n_side_tag as usize + KING_FROM];
        if sq_src != 0 {
            let lpucsq_dst = self.pre_gen.ucsq_king_moves[sq_src as usize];
            for sq_dst in lpucsq_dst {
                if sq_dst != 0 {
                    if self.ucpc_squares[sq_dst as usize] == 0 {
                        res.push(Move::new(
                            self.get_piece_char(KING_TYPE),
                            sq_src as usize,
                            sq_dst,
                        ));
                    }
                }
            }
        }

        // 2. 生成仕(士)的着法
        for i in ADVISOR_FROM..=ADVISOR_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst = self.pre_gen.ucsq_advisor_moves[sq_src as usize];
                for sq_dst in lpucsq_dst {
                    if sq_dst != 0 {
                        if self.ucpc_squares[sq_dst as usize] == 0 {
                            res.push(Move::new(
                                self.get_piece_char(ADVISOR_TYPE),
                                sq_src as usize,
                                sq_dst,
                            ));
                        }
                    }
                }
            }
        }

        // 3. 生成相(象)的着法
        for i in BISHOP_FROM..=BISHOP_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst = self.pre_gen.ucsq_bishop_moves[sq_src as usize];
                let lpucsq_pin = self.pre_gen.ucsq_bishop_pins[sq_src as usize];
                let mut lpucsq_pin_iter = lpucsq_pin.iter();
                for sq_dst in lpucsq_dst {
                    if sq_dst != 0 {
                        if self.ucpc_squares[*lpucsq_pin_iter.next().unwrap() as usize] == 0
                            && self.ucpc_squares[sq_dst as usize] == 0
                        {
                            res.push(Move::new(
                                self.get_piece_char(BISHOP_TYPE),
                                sq_src as usize,
                                sq_dst,
                            ));
                        }
                    }
                }
            }
        }

        // 4. 生成马的着法
        for i in KNIGHT_FROM..=KNIGHT_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst = self.pre_gen.ucsq_knight_moves[sq_src as usize];
                let lpucsq_pin = self.pre_gen.ucsq_knight_pins[sq_src as usize];
                for (sq_dst, sq_pin) in lpucsq_dst.iter().zip(lpucsq_pin) {
                    if *sq_dst != 0
                        && self.ucpc_squares[sq_pin as usize] == 0
                        && self.ucpc_squares[*sq_dst as usize] == 0
                    {
                        res.push(Move::new(
                            self.get_piece_char(KNIGHT_TYPE),
                            sq_src as usize,
                            *sq_dst,
                        ));
                    }
                }
            }
        }

        // 5. 生成车和炮的着法，没有必要判断是否吃到本方棋子
        for i in ROOK_FROM..=CANNON_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let x = file_x(sq_src as usize);
                let y = rank_y(sq_src as usize);
                let lpsmv = self.rank_move(x, y);
                let mut sq_dst = lpsmv.ucNonCap[0] + rank_disp(y);
                let piece_type = if i < CANNON_FROM {
                    ROOK_TYPE
                } else {
                    CANNON_TYPE
                };
                while sq_dst != sq_src && sq_dst > 0 {
                    let mv = Move::new(self.get_piece_char(piece_type), sq_src as usize, sq_dst);
                    res.push(mv);
                    sq_dst -= 1;
                }
                sq_dst = lpsmv.ucNonCap[1] + rank_disp(y);
                while sq_dst != sq_src && sq_dst > 0 {
                    let mv = Move::new(self.get_piece_char(piece_type), sq_src as usize, sq_dst);
                    res.push(mv);
                    sq_dst += 1;
                }
                let lpsmv = self.file_move(x, y);
                sq_dst = lpsmv.ucNonCap[0] + file_disp(x);
                while sq_dst != sq_src && sq_dst > 0 {
                    let mv = Move::new(self.get_piece_char(piece_type), sq_src as usize, sq_dst);
                    res.push(mv);
                    sq_dst -= 16;
                }
                sq_dst = lpsmv.ucNonCap[1] + file_disp(x);
                while sq_dst != sq_src {
                    let mv = Move::new(self.get_piece_char(piece_type), sq_src as usize, sq_dst);
                    res.push(mv);
                    sq_dst += 16;
                }
            }
        }

        // 6. 生成兵(卒)的着法
        for i in PAWN_FROM..=PAWN_TO {
            let sq_src = self.ucsq_pieces[n_side_tag as usize + i];
            if sq_src != 0 {
                let lpucsq_dst =
                    self.pre_gen.ucsq_pawn_moves[self.sd_player as usize][sq_src as usize];
                for sq_dst in lpucsq_dst {
                    if sq_dst != 0 {
                        if self.ucpc_squares[sq_dst as usize] == 0 {
                            res.push(Move::new(
                                self.get_piece_char(PAWN_TYPE),
                                sq_src as usize,
                                sq_dst,
                            ));
                        }
                    }
                }
            }
        }
        res
    }
}

fn fen_piece(n_arg: char) -> usize {
    match n_arg {
        'K' => KING_TYPE,
        'Q' | 'A' => ADVISOR_TYPE,
        'B' | 'E' => BISHOP_TYPE,
        'N' | 'H' => KNIGHT_TYPE,
        'R' => ROOK_TYPE,
        'C' => CANNON_TYPE,
        'P' => PAWN_TYPE,
        _ => 7,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::position::{away_half, rank_y, square_forward, Position};

    #[test]
    fn test_fen() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let board = Position::from_fen(fen);
        let fen2 = Position::to_fen(&board);
        assert_eq!(&fen[0..61], &fen2[0..61]);
        //let moves = board.generate_all_moves(&Side::Red);
    }

    #[test]
    fn test_gen_cap_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(fen);
        let moves = position.gen_cap_moves();
        assert_eq!(moves.len(), 2);
    }

    #[test]
    fn test_gen_nocap_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(fen);
        let moves = position.gen_nocap_moves();
        for mv in moves.iter().filter(|mv| mv.piece == 'C' || mv.piece == 'R') {
            println!("{}", &mv.to_string());
        }

        assert_eq!(moves.len(), 42);
    }
    #[test]
    fn test_gen_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(fen);
        let moves = position.gen_legal_moves();
        assert_eq!(moves.len(), 44);
    }
    #[test]
    fn test_piece_loc() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(fen);
        let piece_locs = position.piece_loc();
        for i in piece_locs {
            println!("{} at {:x}", i.0, i.1);
        }
    }
    #[test]
    fn test_as() {
        let x = -1;
        let y = x as usize;
        assert_ne!(y, 1);
    }

    #[rstest]
    #[case(0x33, 0, 0xc3)]
    #[case(0x33, 1, 0xc3)]
    fn test_square_forward(#[case] src: i32, #[case] sd: i32, #[case] expected: i32) {
        let x = square_forward(src, sd);
        assert_eq!(x, expected);
    }

    #[rstest]
    #[case(0x33, 0, false)]
    #[case(0x33, 1, true)]
    fn test_away_half(#[case] src: i32, #[case] sd: i32, #[case] expected: bool) {
        let x = away_half(src, sd);
        assert_eq!(x, expected);
    }

    #[rstest]
    #[case(51, 3)]
    #[case(0x33, 3)]
    fn test_rank_y(#[case] src: usize, #[case] expected: usize) {
        let x = rank_y(src);
        assert_eq!(x, expected);
    }
}
