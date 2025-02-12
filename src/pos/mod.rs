#![allow(unused_variables)]
#![allow(unused, dead_code)]
#![allow(warnings)]

pub mod fen;
mod game;
pub(crate) mod moves;
mod piece;
pub mod position;
mod pregen;

pub const RANK_TOP: usize = 3;
pub const RANK_BOTTOM: usize = 12;
pub const FILE_LEFT: usize = 3;
pub const FILE_CENTER: usize = 7;
pub const FILE_RIGHT: usize = 11;

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
const CN_PIECE_TYPES: [u8; 48] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 6,
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 6,
];

const CSZ_PIECE_BYTES: &str = "KABNRCP";

fn piece_char(pt: usize) -> char {
    CSZ_PIECE_BYTES.chars().nth(pt).unwrap()
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
    CN_PIECE_TYPES[pc]
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

pub(crate) const KING_FROM: usize = 0;
pub(crate) const ADVISOR_FROM: usize = 1;
pub(crate) const ADVISOR_TO: usize = 2;
pub(crate) const BISHOP_FROM: usize = 3;
pub(crate) const BISHOP_TO: usize = 4;
pub(crate) const KNIGHT_FROM: usize = 5;
pub(crate) const KNIGHT_TO: usize = 6;
pub(crate) const ROOK_FROM: usize = 7;
pub(crate) const ROOK_TO: usize = 8;
pub(crate) const CANNON_FROM: usize = 9;
pub(crate) const CANNON_TO: usize = 10;
pub(crate) const PAWN_FROM: usize = 11;
pub(crate) const PAWN_TO: usize = 15;

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

pub(crate) fn side_tag(sd: usize) -> usize {
    16 + (sd << 4)
}
use serde::{Deserialize, Serialize};
use toto::Toi32;

// By default, player is Red, and computer is Black.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[derive(Clone, Copy, Eq, Hash)]
pub(crate) enum ChessPlayer {
    Red,
    Black,
}

fn side_tag2(sd: &ChessPlayer) -> u8 {
    match sd {
        ChessPlayer::Red => 16,
        ChessPlayer::Black => 32,
    }
}

fn opp_side_tag(sd: usize) -> usize {
    32 - (sd << 4)
}

fn opp_side_tag2(sd: &ChessPlayer) -> usize {
    match sd {
        ChessPlayer::Red => 32,
        ChessPlayer::Black => 16,
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
