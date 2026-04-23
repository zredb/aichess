use std::fmt::{Display, Formatter};
use crate::pos::{file_x, rank_y};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub piece: char,
    pub from: usize,
    pub to: u8,
}

const BOARD_FILES: usize = 9;
const BOARD_RANKS: usize = 10;
const BOARD_SIZE: usize = BOARD_FILES * BOARD_RANKS;
const FILE_LEFT: usize = 3;
const FILE_RIGHT: usize = 11;
const RANK_TOP: usize = 3;
const RANK_BOTTOM: usize = 12;

impl Move {
    pub(crate) fn new(p: char, f: usize, t: u8) -> Move {
        Move {
            piece: p,
            from: f,
            to: t,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // 将Move转换为中文纵线格式，如"马二进三"
        let chinese_notation = self.to_chinese_notation();
        write!(f, "{}", chinese_notation)
    }
}

impl Move {
    /// 将着法转换为中文纵线格式 (如 "马二进三")
    pub fn to_chinese_notation(&self) -> String {
        // 获取棋子的中文字符
        let piece_char = self.piece_to_chinese(self.piece);
        
        // 计算起始和目标位置的棋盘坐标
        let from_file = file_x(self.from);  // 横坐标 (FILE_LEFT=3 到 FILE_RIGHT=11)
        let from_rank = rank_y(self.from);  // 纵坐标 (RANK_TOP=3 到 RANK_BOTTOM=12)
        let to_file = file_x(self.to as usize);
        let to_rank = rank_y(self.to as usize);
        
        // 判断是红方还是黑方
        let is_red = self.piece.is_ascii_uppercase();
        
        // 计算纵线编号 (红方从右到左1-9, 黑方从左到右1-9)
        let from_file_num = if is_red {
            FILE_RIGHT - from_file + 1  // 红方: 右边是1, 左边是9
        } else {
            from_file - FILE_LEFT + 1   // 黑方: 左边是1, 右边是9
        };
        
        // 将数字转换为中文数字
        let from_file_cn = Self::number_to_chinese(from_file_num);
        
        // 判断移动方向: 进、退、平
        let (direction, target_str) = if from_file == to_file {
            // 纵向移动
            let direction_char = if is_red {
                if to_rank < from_rank { "进" } else { "退" }
            } else {
                if to_rank > from_rank { "进" } else { "退" }
            };
            
            // 对于车、炮、兵(卒)、帅(将)，用目标位置的纵线编号
            // 对于马、相(象)、仕(士)，用移动的步数
            let target = match self.piece.to_ascii_lowercase() {
                'k' | 'r' | 'c' | 'p' => {
                    // 车、炮、兵、帅：使用目标位置的纵线编号
                    let to_file_num = if is_red {
                        FILE_RIGHT - to_file + 1
                    } else {
                        to_file - FILE_LEFT + 1
                    };
                    Self::number_to_chinese(to_file_num)
                },
                _ => {
                    // 马、相、仕：使用移动的步数(行数差)
                    let steps = if is_red {
                        from_rank - to_rank
                    } else {
                        to_rank - from_rank
                    };
                    Self::number_to_chinese(steps)
                }
            };
            
            (direction_char, target)
        } else {
            // 横向移动 - 一定是"平"
            let to_file_num = if is_red {
                FILE_RIGHT - to_file + 1
            } else {
                to_file - FILE_LEFT + 1
            };
            ("平", Self::number_to_chinese(to_file_num))
        };
        
        format!("{}{}{}{}", piece_char, from_file_cn, direction, target_str)
    }
    
    /// 将棋子字符转换为中文
    fn piece_to_chinese(&self, piece: char) -> &'static str {
        match piece.to_ascii_lowercase() {
            'k' => {
                if piece.is_ascii_uppercase() { "帅" } else { "将" }
            },
            'a' => {
                if piece.is_ascii_uppercase() { "仕" } else { "士" }
            },
            'b' => {
                if piece.is_ascii_uppercase() { "相" } else { "象" }
            },
            'n' => "马",
            'r' => "车",
            'c' => "炮",
            'p' => {
                if piece.is_ascii_uppercase() { "兵" } else { "卒" }
            },
            _ => "?",
        }
    }
    
    /// 将数字转换为中文数字 (1-10)
    fn number_to_chinese(num: usize) -> &'static str {
        const CHINESE_NUMBERS: [&str; 11] = [
            "", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十"
        ];
        if num >= 1 && num <= 10 {
            CHINESE_NUMBERS[num]
        } else {
            "?"
        }
    }
}

fn board_index_from_square(square: usize) -> Option<usize> {
    let file = square & 0x0f;
    let rank = square >> 4;
    if !(FILE_LEFT..=FILE_RIGHT).contains(&file) || !(RANK_TOP..=RANK_BOTTOM).contains(&rank) {
        return None;
    }
    Some((rank - RANK_TOP) * BOARD_FILES + (file - FILE_LEFT))
}

fn square_from_board_index(index: usize) -> usize {
    let rank = index / BOARD_FILES + RANK_TOP;
    let file = index % BOARD_FILES + FILE_LEFT;
    (rank << 4) | file
}

impl From<usize> for Move {
    fn from(value: usize) -> Self {
        let from = value / BOARD_SIZE;
        let to = value % BOARD_SIZE;
        Move::new('?', square_from_board_index(from), square_from_board_index(to) as u8)
    }
}
impl Into<usize> for Move {
    fn into(self) -> usize {
        let from = board_index_from_square(self.from)
            .expect("move source square must be on the 9x10 board");
        let to = board_index_from_square(self.to as usize)
            .expect("move destination square must be on the 9x10 board");
        from * BOARD_SIZE + to
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

const DIGIT_2_WORD: [char; 10] = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];

const PIECE_2_WORD: [[char; 8]; 2] = [
    ['帅', '仕', '相', '马', '车', '炮', '兵', ' '],
    ['将', '士', '象', '马', '车', '炮', '卒', ' '],
];

const DIRECT_2_WORD: [char; 4] = ['进', '平', '退', ' '];

const POS_2_WORD: [char; 10] = ['一', '二', '三', '四', '五', '前', '中', '后', ' ', ' '];

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

#[cfg(test)]
mod tests {
    use super::Move;

    #[test]
    fn move_index_roundtrip_preserves_board_squares() {
        let mv = Move::new('R', 0x33, 0x3b);
        let index: usize = mv.into();
        let decoded = Move::from(index);

        assert_eq!(decoded.from, 0x33);
        assert_eq!(decoded.to, 0x3b);
    }

    #[test]
    fn test_chinese_notation_knight_move() {
        // 马二进三: 红马从纵线2进到纵线3
        // from: file=10 (FILE_RIGHT-2+1=2), rank=?
        // to: file=9 (FILE_RIGHT-3+1=3), rank=?
        // FILE_LEFT=3, FILE_RIGHT=11
        // 红方纵线2: file = FILE_RIGHT - 2 + 1 = 11 - 2 + 1 = 10
        // 红方纵线3: file = FILE_RIGHT - 3 + 1 = 11 - 3 + 1 = 9
        
        // 假设起始位置在红方底线(rank=12), 目标位置前进到rank=10
        let from = (12 << 4) | 10;  // rank=12, file=10
        let to = (10 << 4) | 9;     // rank=10, file=9
        
        let mv = Move::new('N', from, to as u8);
        let notation = format!("{}", mv);
        
        println!("Knight move notation: {}", notation);
        assert_eq!(notation, "马二进三");
    }

    #[test]
    fn test_chinese_notation_rook_horizontal() {
        // 车一平二: 红车从纵线1平移到纵线2
        // 红方纵线1: file = FILE_RIGHT - 1 + 1 = 11
        // 红方纵线2: file = FILE_RIGHT - 2 + 1 = 10
        
        let from = (12 << 4) | 11;  // rank=12, file=11 (纵线1)
        let to = (12 << 4) | 10;    // rank=12, file=10 (纵线2)
        
        let mv = Move::new('R', from, to as u8);
        let notation = format!("{}", mv);
        
        println!("Rook horizontal move notation: {}", notation);
        assert_eq!(notation, "车一平二");
    }

    #[test]
    fn test_chinese_notation_cannon_forward() {
        // 炮二进四: 红炮从纵线2前进4步
        // 红方纵线2: file = 10
        // 前进4步: rank从12到8
        
        let from = (12 << 4) | 10;  // rank=12, file=10
        let to = (8 << 4) | 10;     // rank=8, file=10
        
        let mv = Move::new('C', from, to as u8);
        let notation = format!("{}", mv);
        
        println!("Cannon forward move notation: {}", notation);
        assert_eq!(notation, "炮二进四");
    }
}
