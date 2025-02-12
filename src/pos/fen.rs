use crate::pos::{coord_xy, FILE_LEFT, FILE_RIGHT, RANK_BOTTOM, RANK_TOP};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fen(&'static str);

const PIECE_TYPES: &str = "KABNRCPkabnrcp";

impl Fen {}

pub fn fen2_coords(fen: &str) -> Vec<(char, u8)> {
    let mut piece_locs = vec![];
    let mut i = RANK_TOP;
    let mut j = FILE_LEFT;

    let mut lp_fen: String = fen.split(" ").collect();
    let first_space_index = fen.find(' ').unwrap_or(fen.len());
    let mut first = &fen[..first_space_index]; //取fen的第一段

    for ch in first.chars() {
        if ch == '/' {
            j = FILE_LEFT;
            i += 1;
            if i > RANK_BOTTOM {
                break;
            }
        } else if ch.is_ascii_digit() {
            j += ch.to_digit(10).unwrap() as usize;//如果是数字, 则向前滑动j值
        } else if PIECE_TYPES.contains(ch) {
            if j <= FILE_RIGHT {
                piece_locs.push((ch, coord_xy(j, i) as u8));
            }
            j += 1;
        } else {// 不认识的字符, 直接跳过, 这里是否合适?
            j += 1;
        }
    }
    piece_locs
}