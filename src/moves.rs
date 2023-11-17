use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};


///https://www.xqbase.com/protocol/cchess_move.htm
/*
```
 红方	黑方	字母	相当于国际象棋中的棋子	数字
 帅	将	K	    King(王)	1
 仕	士	A	    Advisor(没有可比较的棋子)	2
 相	象	B[1]	Bishop(象)	3
 马	马	N[2]	Knight(马)	4
 车	车	R	    Rook(车)	5
 炮	炮	C	    Cannon(没有可比较的棋子)	6
 兵	卒	P	    Pawn(兵)	7
 表一　中国象棋棋子代号
 [1] 世界象棋联合会推荐的字母代号为E(Elephant)
 [2] 世界象棋联合会推荐的字母代号为H(Horse)
```
*/
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Move {
    pub(crate) piece: char,
    pub(crate) from: usize,
    pub(crate) to: u8,
}

impl Move {
    pub(crate) fn new(p: char, f: usize, t: u8) -> Move {
        Move {
            piece: p,
            from: f,
            to: t,
        }
    }
}

///  ICCS坐标格式
///  ICCS是中国象棋互联网服务器(Internet Chinese Chess Server)的缩写。
/// 在网络对弈服务器处理着法时，把着法表示成起点和终点的坐标是最方便的
/// ，因此这种格式最早在计算机上使用。
/// 1. H2-E2	(炮二平五)	　	H7-E7	(炮８平５)
/// 2. E2-E6	(炮五进四)	　	D9-E8	(士４进５)
/// 3. H0-G2	(马二进三)	　	H9-G7	(马８进７)
/// 4. B2-E2	(炮八平五)	　	B9-C7	(马２进３)
/// 5. E6-E4	(前炮退二)	　	I9-H9	(车９平８)
/// 6. ……	(如右图)	　	　
/// 在“中国象棋通用引擎协议”(UCCI协议)中，坐标格式得到进一步简化，例如H2-E2记作h2e2，把符号限制在一个32位数据中，处理起来速度更快。	　
struct Iccs(String);

impl Display for Iccs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// impl From<Move> for Iccs {
//     fn from(mv: Move) -> Self {
//         // 0~9
//         let row_src: usize = INDEX_ROW[mv.from];
//         // 0~8
//         let col_src: usize = INDEX_COLUMN[mv.from];
//
//         // 0~9
//         let row_dst: usize = INDEX_ROW[mv.to];
//         // 0~8
//         let col_dst: usize = INDEX_COLUMN[mv.to];
//         let iccs = format!("{}{}{}{}", column_to_char(col_src), row_src, column_to_char(col_dst), row_dst);
//         Iccs(iccs)
//     }
// }

fn column_to_char(col: usize) -> char {
    match col {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        8 => 'i',
        _ => panic!("error col number"),
    }
}

const CC_DIRECT_2_BYTE: [char; 4] = ['+', '.', '-', ' '];

const CC_POS_2_BYTE: [char; 12] = ['a', 'b', 'c', 'd', 'e', '+', '.', '-', ' ', ' ', ' ', ' '];

const DIGIT_2_WORD: [char; 10] = [
    '一', '二', '三', '四', '五',
    '六', '七', '八', '九', '十',
];

const PIECE_2_WORD: [[char; 8]; 2] = [
    [
        '帅', '仕', '相', '马', '车', '炮', '兵', ' ',
    ],
    [
        '将', '士', '象', '马', '车', '炮', '卒', ' ',
    ],
];

const DIRECT_2_WORD: [char; 4] = ['进', '平', '退', ' '];

const POS_2_WORD: [char; 10] = [
    '一', '二', '三', '四', '五',
    '前', '中', '后', ' ', ' ',
];

///中文纵线格式
struct FixFile {}

///着法表示成中文纵线格式
// fn to_cvlf(board: &Board, mv: &Move) -> String {
//     "".into()
// }
//
// ///着法表示成数字纵线格式
// fn to_dvlf(board: &Board, mv: &Move) -> String {
//     // 0~9
//     let row_src: usize = INDEX_ROW[mv.from];
//     // 0~8
//     let col_src: usize = INDEX_COLUMN[mv.from];
//
//     // 0~9
//     let row_dst: usize = INDEX_ROW[mv.to];
//     // 0~8
//     let col_dst: usize = INDEX_COLUMN[mv.to];
//     let piece=mv.piece;
//     match mv.piece {
//         RED_ADVISER => {
//             let aa=board.search_piece_locations(piece);
//
//         },
//     }
//
//     "".into()
// }

///WXF纵线格式
struct Wfx(String);

impl Display for Wfx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Move> for Wfx {
    fn from(value: Move) -> Self {
        todo!()
    }
}


