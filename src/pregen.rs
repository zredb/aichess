use std::mem::size_of;
use rand::random;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;
use toto::{Toi32, Tou8};
use crate::{away_half, bishop_pin, file_disp, FILE_LEFT, file_x, in_board, in_fort, knight_pin, rank_disp, RANK_TOP, rank_y, same_half, square_forward};


// 借助“位行”和“位列”生成车炮着法的预置结构
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct SlideMove {
    // 不吃子能走到的最大一格/最小一格
    pub(crate) uc_non_cap: [u8; 2],
    // 车吃子能走到的最大一格/最小一格
    pub(crate) uc_rook_cap: [u8; 2],
    // 炮吃子能走到的最大一格/最小一格
    pub(crate) uc_cannon_cap: [u8; 2],
    // 超级炮(隔两子吃子)能走到的最大一格/最小一格
    pub(crate) uc_super_cap: [u8; 2],
}

//smv
impl SlideMove {
    fn new(init: u8) -> Self {
        SlideMove {
            uc_non_cap: [init; 2],
            uc_rook_cap: [init; 2],
            uc_cannon_cap: [init; 2],
            uc_super_cap: [init; 2],
        }
    }
}

// 借助“位行”和“位列”判断车炮着法合理性的预置结构
#[derive(Copy, Clone, Debug, Default, PartialEq,Eq, Hash)]
pub(crate) struct SlideMask {
    w_non_cap: u16,
    w_rook_cap: u16,
    w_cannon_cap: u16,
    w_super_cap: u16,
}// sms

#[derive(Debug, Default, Clone, Copy , PartialEq, Eq, Hash)]
pub(crate) struct Zobrist {
    dw_key: u32,
    dw_lock0: u32,
    dw_lock1: u32,
}

const CN_KING_MOVE_TAB: [i8; 4] = [-0x10, -0x01, 0x01, 0x10];
const CN_ADVISOR_MOVE_TAB: [i8; 4] = [-0x11, -0x0f, 0x0f, 0x11];
const CN_BISHOP_MOVE_TAB: [i8; 4] = [-0x22, -0x1e, 0x1e, 0x22];
const CN_KNIGHT_MOVE_TAB: [i8; 8] = [-0x21, -0x1f, -0x12, -0x0e, 0x0e, 0x12, 0x1f, 0x21];


impl Zobrist {
    pub(crate) fn init_rc4() -> Self {
        Zobrist {
            dw_key: random(),
            dw_lock0: random(),
            dw_lock1: random(),
        }
    }

    pub(crate) fn xor(&mut self, zobr: &Zobrist) {
        self.dw_key ^= zobr.dw_key;
        self.dw_lock0 ^= zobr.dw_lock0;
        self.dw_lock1 ^= zobr.dw_lock1;
    }

    fn xor2(&mut self, zobr1: &Zobrist, zobr2: &Zobrist) {
        self.dw_key ^= zobr1.dw_key ^ zobr2.dw_key;
        self.dw_lock0 ^= zobr1.dw_lock0 ^ zobr2.dw_lock0;
        self.dw_lock1 ^= zobr1.dw_lock1 ^ zobr2.dw_lock1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PreGen {

    zobr_player: Zobrist,

    pub(crate) zobr_table: [[Zobrist; 14]; 256],
    //以“位”的形式记录棋盘上某一行所有的格子的状态(仅仅指是否有子)，就称为“位行”(BitRank)，与之对应的是“位列”(BitFile)，棋盘结构应该包含10个位行和9个位列

    pub(crate) w_bit_rank_mask: [u16; 256],
    // 每个格子的位行的屏蔽位(10位就够了)

    pub(crate) w_bit_file_mask: [u16; 256], // 每个格子的位列的屏蔽位()

    /* 借助“位行”和“位列”生成车炮着法和判断车炮着法合理性的预置数组
    *
    * “位行”和“位列”技术是ElephantEye的核心技术，用来处理车和炮的着法生成、将军判断和局面分析。
    * 以初始局面红方右边的炮在该列的行动为例，首先必须知道该列的“位列”，即"1010000101b"，
    * ElephantEye有两种预置数组，即"...MoveTab"和"...MaskTab"，用法分别是：
    * 一、如果要知道该子向前吃子的目标格(起始行是2，目标行是9)，那么希望查表就能知道这个格子，
    * 　　预先生成一个数组"FileMoveTab_CannonCap[10][1024]"，使得"FileMoveTab_CannonCap[2][1010000101b] == 9"就可以了。
    * 二、如果要判断该子能否吃到目标格(同样以起始格是2，目标格是9为例)，那么需要知道目标格的位列，即"0000000001b"，
    * 　　只要把"...MoveTab"的格子以“屏蔽位”的形式重新记作数组"...MaskTab"就可以了，用“与”操作来判断能否吃到目标格，
    * 　　通常一个"...MaskTab"单元会包括多个屏蔽位，判断能否吃到同行或同列的某个格子时，只需要做一次判断就可以了。
    */
    pub(crate) smv_file_move_tab: [[SlideMove; 1024]; 10],
    // 1024=2^10, 可以表示某列中所有行(10)是否有棋子的所有状态,例如1010000101b
    pub(crate) sms_file_mask_tab: [[SlideMask; 1024]; 10],
    pub(crate) smv_rank_move_tab: [[SlideMove; 512]; 9],
    pub(crate) sms_rank_mask_tab: [[SlideMask; 512]; 9],

    /* 其余棋子(不适合用“位行”和“位列”)的着法预生成数组
    * 这部分数组是真正意义上的“着法预生成”数组，可以根据某个棋子的起始格直接查数组，得到所有的目标格。
    * 使用数组时，可以根据起始格来确定一个指针"g_...Moves[Square]"，这个指针指向一系列目标格，以0结束。
    * 为了对齐地址，数组[256][n]中n总是4的倍数，而且必须大于n(因为数组包括了结束标识符0)，除了象眼和马腿数组。
    */
    pub(crate) ucsq_king_moves: [[u8; 4]; 256],
    pub(crate) ucsq_advisor_moves: [[u8; 4]; 256],
    pub(crate) ucsq_bishop_moves: [[u8; 4]; 256],
    pub(crate) ucsq_bishop_pins: [[u8; 4]; 256],
    pub(crate) ucsq_knight_moves: [[u8; 8]; 256],
    pub(crate) ucsq_knight_pins: [[u8; 8]; 256],
    pub(crate) ucsq_pawn_moves: [[[u8; 4]; 256]; 2],
}

// impl Serialize for PreGen {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//         let size=size_of<PreGen>();
//         let mut s = serializer.serialize_struct("PreGen", 3)?;
//         let serialized = serde_json::to_string(&self.w_bit_rank_mask).unwrap();
//         s.serialize_field("w_bit_rank_mask", &self.w_bit_rank_mask)?;
//         s.serialize_field("age", &self.age)?;
//         s.serialize_field("phones", &self.phones)?;
//         s.end()
//     }
// }

impl PreGen {
    pub(crate) fn new() -> Self {
        // 首先初始化Zobrist键值表
        let mut zobr_table: [[Zobrist; 14]; 256] = [[Zobrist::default(); 14]; 256];

        for i in 0..256 {
            for j in 0..14 {
                zobr_table[i][j] = Zobrist::init_rc4();
            }
        }

        let mut w_bit_rank_mask: [u16; 256] = [0; 256];
        let mut w_bit_file_mask: [u16; 256] = [0; 256];
        // 然后初始化屏蔽位行和屏蔽位列
        // 注：位行和位列不包括棋盘以外的位，所以就会频繁使用"+/- RANK_TOP/FILE_LEFT"
        for sq_src in 0..256usize {
            if in_board(sq_src as i32) {
                w_bit_rank_mask[sq_src] = 1 << (file_x(sq_src) - FILE_LEFT);
                w_bit_file_mask[sq_src] = 1 << (rank_y(sq_src) - RANK_TOP);
            }
        }

        let mut smv_rank_move_tab = [[SlideMove::default(); 512]; 9];
        let mut smv_file_move_tab = [[SlideMove::default(); 1024]; 10];
        let mut sms_rank_mask_tab = [[SlideMask::default(); 512]; 9];
        let mut sms_file_mask_tab = [[SlideMask::default(); 1024]; 10];
        //生成预置数组, 用于快速查询: https://www.xqbase.com/computer/eleeye_struct.htm
        // 如果没有预置数组,马的目标位置应如下计算:DstSq = SrcSq + cnKnightMoveTab[j]; j=0..8
        // 有了预置数组, DstSq = ucsq_knight_moves[SrcSq][j]; //ucsqKnightMoves只需初始化时计算一次,用内存换时间;

        // 然后生成车炮横向的预置数组
        for i in 0..9 {//列号
            for j in 0..512 {
                // 初始化借助于“位行”的车和炮的着法预生成数组，包括以下几个步骤：
                // 1. 初始化临时变量"SlideMove"，假设没有着法，用起始格填充
                let p = (i + FILE_LEFT) as u8;
                let mut smv = SlideMove::new(p);//假设没有着法，用起始格填充
                let mut sms = SlideMask::default();

                // 2. 考虑向右移动的目标格，填充...[0]，
                // 提示：smv.xxx[0]表示最大一格，向右移动和下移动都用[0]，反之亦然; smv.xxx[1]表示最小一格,向左移动和向上移动都用[1]

                let mut right = i + 1usize;
                for k in right..=8 {
                    right += 1;
                    if j & (1 << k) != 0 {
                        smv.uc_rook_cap[0] = file_disp(k + FILE_LEFT);
                        sms.w_rook_cap |= 1 << k;
                        break;
                    }

                    smv.uc_non_cap[0] = file_disp(k + FILE_LEFT);
                    sms.w_non_cap |= 1 << k;
                }

                for k in right..=8 {
                    right += 1;
                    if j & (1 << k) != 0 {
                        smv.uc_cannon_cap[0] = file_disp(k + FILE_LEFT);
                        sms.w_cannon_cap |= 1 << k;
                        break;
                    }
                }
                for k in right..=8 {
                    right += 1;
                    if j & (1 << k) != 0 {
                        smv.uc_super_cap[0] = file_disp(k + FILE_LEFT);
                        sms.w_super_cap |= 1 << k;
                        break;
                    }
                }

                // 3. 考虑向左移动的目标格，填充...[1]
                let mut left = i as i32 - 1;
                if left >= 0 {
                    for k in (0..=left as usize).rev() {
                        left -= 1;
                        if j & (1 << k) != 0 {
                            smv.uc_rook_cap[1] = file_disp(k + FILE_LEFT);
                            sms.w_rook_cap |= 1 << k;
                            break;
                        }
                        smv.uc_non_cap[1] = file_disp(k + FILE_LEFT);
                        sms.w_non_cap |= 1 << k;
                    }
                }
                if left >= 0 {
                    for k in (0..=left as usize).rev() {
                        left -= 1;
                        if j & (1 << k) != 0 {
                            smv.uc_cannon_cap[1] = file_disp(k + FILE_LEFT);
                            sms.w_cannon_cap |= 1 << k;
                            break;
                        }
                    }
                }
                if left >= 0 {
                    for k in (0..=left as usize).rev() {
                        left -= 1;
                        if j & (1 << k) != 0 {
                            smv.uc_super_cap[1] = file_disp(k + FILE_LEFT);
                            sms.w_super_cap |= 1 << k;
                            break;
                        }
                    }
                }

                // 4. 为"smv"和"sms"的值作断言
                assert_bound_2(3, smv.uc_non_cap[1], smv.uc_non_cap[0], 11);
                assert_bound_2(3, smv.uc_rook_cap[1], smv.uc_rook_cap[0], 11);
                assert_bound_2(3, smv.uc_cannon_cap[1], smv.uc_cannon_cap[0], 11);
                assert_bound_2(3, smv.uc_super_cap[1], smv.uc_super_cap[0], 11);
                assert_bitrank(sms.w_non_cap);
                assert_bitrank(sms.w_rook_cap);
                assert_bitrank(sms.w_cannon_cap);
                assert_bitrank(sms.w_super_cap);
                // 5. 将临时变量"smv"和"sms"拷贝到着法预生成数组中
                smv_rank_move_tab[i][j] = smv;
                sms_rank_mask_tab[i][j] = sms;
            }
        }
        // 然后生成车炮纵向的预置数组
        for i in 0..10 {
            for j in 0..1024 {
                // 初始化借助于“位列”的车和炮的着法预生成数组，包括以下几个步骤：
                // 1. 初始化临时变量"SlideMove"，假设没有着法，用起始格填充
                let p = ((i + RANK_TOP) * 16) as u8;
                let mut smv = SlideMove::new(p);//假设没有着法，用起始格填充
                let mut sms = SlideMask::default();

                // 2. 考虑向下移动的目标格，填充...[0]
                // 提示：smv.xxx[0]表示最大一格，向右移动和下移动都用[0]，反之亦然; smv.xxx[1]表示最小一格,向左移动和向上移动都用[1]
                let mut down = i + 1usize;
                for k in down..=9 {
                    down += 1;
                    if j & (1 << k) != 0 {
                        smv.uc_rook_cap[0] = rank_disp(k + RANK_TOP);
                        sms.w_rook_cap |= 1 << k;
                        break;
                    }

                    smv.uc_non_cap[0] = rank_disp(k + RANK_TOP);
                    sms.w_non_cap |= 1 << k;
                }

                for k in down..=9 {
                    down += 1;
                    if j & (1 << k) != 0 {
                        smv.uc_cannon_cap[0] = rank_disp(k + RANK_TOP);
                        sms.w_cannon_cap |= 1 << k;
                        break;
                    }
                }

                for k in down..=9 {
                    down += 1;
                    if j & (1 << k) != 0 {
                        smv.uc_super_cap[0] = rank_disp(k + RANK_TOP);
                        sms.w_super_cap |= 1 << k;
                        break;
                    }
                }
                // 3. 考虑向上移动的目标格，填充...[1]
                let mut up = i as i32 - 1;
                if up >= 0 {
                    for k in (0..=up as usize).rev() {
                        up -= 1;
                        if j & (1 << k) != 0 {
                            smv.uc_rook_cap[1] = rank_disp(k + RANK_TOP);
                            sms.w_rook_cap |= 1 << k;
                            break;
                        }
                        smv.uc_non_cap[1] = rank_disp(k + RANK_TOP);
                        sms.w_non_cap |= 1 << k;
                    }
                }

                if up >= 0 {
                    for k in (0..=up as usize).rev() {
                        up -= 1;
                        if j & (1 << k) != 0 {
                            smv.uc_cannon_cap[1] = rank_disp(k + RANK_TOP);
                            sms.w_cannon_cap |= 1 << k;
                            break;
                        }
                    }
                }
                if up >= 0 {
                    for k in (0..=up as usize).rev() {
                        up -= 1;
                        if j & (1 << k) != 0 {
                            smv.uc_super_cap[1] = rank_disp(k + RANK_TOP);
                            sms.w_super_cap |= 1 << k;
                            break;
                        }
                    }
                }

                // 4. 为"smv"和"sms"的值作断言
                assert_bound_2(3, smv.uc_non_cap[1] >> 4, smv.uc_non_cap[0] >> 4, 12);
                assert_bound_2(3, smv.uc_rook_cap[1] >> 4, smv.uc_rook_cap[0] >> 4, 12);
                assert_bound_2(3, smv.uc_cannon_cap[1] >> 4, smv.uc_cannon_cap[0] >> 4, 12);
                assert_bound_2(3, smv.uc_super_cap[1] >> 4, smv.uc_super_cap[0] >> 4, 12);
                assert_bitfile(sms.w_non_cap);
                assert_bitfile(sms.w_rook_cap);
                assert_bitfile(sms.w_cannon_cap);
                assert_bitfile(sms.w_super_cap);

                // 5. 将临时变量"smv"和"sms"拷贝到着法预生成数组中
                smv_file_move_tab[i][j] = smv;
                sms_file_mask_tab[i][j] = sms;
            }
        }
        let mut ucsq_king_moves: [[u8; 4]; 256] = [[0u8; 4]; 256];
        let mut ucsq_advisor_moves: [[u8; 4]; 256] = [[0u8; 4]; 256];
        let mut ucsq_bishop_moves: [[u8; 4]; 256] = [[0u8; 4]; 256];
        let mut ucsq_bishop_pins: [[u8; 4]; 256] = [[0u8; 4]; 256];
        let mut ucsq_knight_moves: [[u8; 8]; 256] = [[0u8; 8]; 256];
        let mut ucsq_knight_pins: [[u8; 8]; 256] = [[0u8; 8]; 256];
        let mut ucsq_pawn_moves: [[[u8; 4]; 256]; 2] = [[[0u8; 4]; 256]; 2];
        for sq_src in 0..256 {
            if in_board(sq_src) {
                // 生成帅(将)的着法预生成数组
                for i in 0..4 {
                    let sq_dst = sq_src + CN_KING_MOVE_TAB[i].to_i32();
                    if in_fort(sq_dst) {
                        ucsq_king_moves[sq_src as usize][i] = sq_dst.to_u8();
                    }
                }

                // 生成仕(士)的着法预生成数组
                for i in 0..4 {
                    let sq_dst = sq_src as i32 + CN_ADVISOR_MOVE_TAB[i].to_i32();
                    if sq_dst >= 0 && in_fort(sq_dst) {
                        ucsq_advisor_moves[sq_src as usize][i] = sq_dst.to_u8();
                    }
                }

                // 生成相(象)的着法预生成数组，包括象眼数组
                for i in 0..4 {
                    let sq_dst = sq_src + CN_BISHOP_MOVE_TAB[i].to_i32();
                    if in_board(sq_dst) && same_half(sq_src, sq_dst) {
                        ucsq_bishop_moves[sq_src as usize][i] = sq_dst.to_u8();
                        ucsq_bishop_pins[sq_src as usize][i] = bishop_pin(sq_src, sq_dst);
                    }
                }

                // 生成马的着法预生成数组，包括马腿数组
                for i in 0..8 {
                    let sq_dst = sq_src + CN_KNIGHT_MOVE_TAB[i].to_i32();
                    if in_board(sq_dst) {
                        ucsq_knight_moves[sq_src as usize][i] = sq_dst.to_u8();
                        ucsq_knight_pins[sq_src as usize][i] = knight_pin(sq_src, sq_dst);
                    }
                }

                // 生成兵(卒)的着法预生成数组, 即计算好棋盘上所有256个位置上为兵(卒)时, 它可以到的位置;
                for i in 0..2 {// 0:兵(上方),1:卒两种
                    let mut n = 0;
                    let mut sq_dst = square_forward(sq_src, i);
                    sq_dst = sq_src + if i == 0 { -16 } else { 16 };
                    if in_board(sq_dst) {
                        ucsq_pawn_moves[i as usize][sq_src as usize][n] = sq_dst.to_u8();
                    }
                    if away_half(sq_src, i) { //过了河
                        for j in -1..=1 {
                            sq_dst = sq_src + j;
                            n = (j + 1) as usize;
                            if in_board(sq_dst) {
                                ucsq_pawn_moves[i as usize][sq_src as usize][n] = sq_dst.to_u8();
                            }
                        }
                    }
                }
            }
        }


        let pregen = PreGen {
            zobr_player: Zobrist::init_rc4(),
            zobr_table,
            w_bit_rank_mask,
            w_bit_file_mask,
            smv_rank_move_tab,
            smv_file_move_tab,
            sms_rank_mask_tab,
            sms_file_mask_tab,
            ucsq_king_moves,
            ucsq_advisor_moves,
            ucsq_bishop_moves,
            ucsq_bishop_pins,
            ucsq_knight_moves,
            ucsq_knight_pins,
            ucsq_pawn_moves,
        };
        pregen
    }
}


fn assert_bound_2(a: u8, b: u8, c: u8, d: u8) {
    assert!(a <= b && b <= c && c <= d);
}

const fn assert_bitrank(w: u16) {
    assert!(w < (1 << 9));
}

const fn assert_bitfile(w: u16) {
    assert!(w < (1 << 10));
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use serde::Serialize;
    use crate::{file_x, in_board, rank_y};

    use crate::pregen::PreGen;

    #[test]
    fn test_pre_gen() {
        let pre_gen = PreGen::new();

        for i in 0..256 {
            let x = file_x(i);
            let y = rank_y(i);

            if in_board(i as i32) {
                match x {
                    3 => assert_eq!(pre_gen.w_bit_rank_mask[i], 1),
                    4 => assert_eq!(pre_gen.w_bit_rank_mask[i], 2),
                    5 => assert_eq!(pre_gen.w_bit_rank_mask[i], 4),
                    6 => assert_eq!(pre_gen.w_bit_rank_mask[i], 8),
                    7 => assert_eq!(pre_gen.w_bit_rank_mask[i], 16),
                    8 => assert_eq!(pre_gen.w_bit_rank_mask[i], 32),
                    9 => assert_eq!(pre_gen.w_bit_rank_mask[i], 64),
                    10 => assert_eq!(pre_gen.w_bit_rank_mask[i], 128),
                    11 => assert_eq!(pre_gen.w_bit_rank_mask[i], 256),
                    12 => assert_eq!(pre_gen.w_bit_rank_mask[i], 512),
                    _ => {}
                };
                match y {
                    3 => assert_eq!(pre_gen.w_bit_file_mask[i], 1),
                    4 => assert_eq!(pre_gen.w_bit_file_mask[i], 2),
                    5 => assert_eq!(pre_gen.w_bit_file_mask[i], 4),
                    6 => assert_eq!(pre_gen.w_bit_file_mask[i], 8),
                    7 => assert_eq!(pre_gen.w_bit_file_mask[i], 16),
                    8 => assert_eq!(pre_gen.w_bit_file_mask[i], 32),
                    9 => assert_eq!(pre_gen.w_bit_file_mask[i], 64),
                    10 => assert_eq!(pre_gen.w_bit_file_mask[i], 128),
                    11 => assert_eq!(pre_gen.w_bit_file_mask[i], 256),
                    12 => assert_eq!(pre_gen.w_bit_file_mask[i], 512),
                    _ => {}
                };
            }
            let mut s = String::new();
            let mut c = 0;
            for j in 0..9 {
                s.push_str( &format!("Rank:{j}--------------NonCap------RookCap-----CannonCap---SuperCap-----\n"));
                let mvs = pre_gen.smv_rank_move_tab[j];
                let masks = pre_gen.sms_rank_mask_tab[j];
                c = 0;
                for (mv, mask) in mvs.iter().zip(masks) {
                    s.push_str( &format!("SlideMove[{c:4}]: \t[{:x},{:x}], \t\t[{:x},{:x}], \t\t[{:x},{:x}], \t\t[{:x},{:x}]\n", mv.uc_non_cap[0], mv.uc_non_cap[1], mv.uc_rook_cap[0], mv.uc_rook_cap[1], mv.uc_cannon_cap[0], mv.uc_cannon_cap[1], mv.uc_super_cap[0], mv.uc_super_cap[1], ));
                    s.push_str( &format!("SlideMask[{c:4}]: \t[{:x}], \t\t[{:x}], \t\t[{:x}], \t\t[{:x}]\n", mask.w_non_cap, mask.w_rook_cap, mask.w_cannon_cap, mask.w_super_cap));
                    c += 1;
                }
            }
            for i in 0..10 {
                s.push_str( &format!("File:{i}--------------NonCap------RookCap-----CannonCap---SuperCap-----\n"));
                let mvs = pre_gen.smv_file_move_tab[i];
                let masks = pre_gen.sms_file_mask_tab[i];
                c = 0;
                for (mv, mask) in mvs.iter().zip(masks) {
                    s.push_str( &format!("SlideMove[{c:4}]: \t[{:x},{:x}], \t\t[{:x},{:x}], \t\t[{:x},{:x}], \t\t[{:x},{:x}]\n", mv.uc_non_cap[0], mv.uc_non_cap[1], mv.uc_rook_cap[0], mv.uc_rook_cap[1], mv.uc_cannon_cap[0], mv.uc_cannon_cap[1], mv.uc_super_cap[0], mv.uc_super_cap[1], ));
                    s.push_str( &format!("SlideMask[{c:4}]: \t[{:x}], \t\t[{:x}], \t\t[{:x}], \t\t[{:x}]\n", mask.w_non_cap, mask.w_rook_cap, mask.w_cannon_cap, mask.w_super_cap));
                    c += 1;
                }
            }
            let right=fs::read_to_string("./src/tests/data/right_smv_sms.txt").unwrap();
            assert_eq!( s, right);// 验证生成的车炮预置数组是否正确,right_smv_sms.txt是经过验证的正确结果

            //let x=pre_gen.ucsq_king_moves.serialize(/* serializer */);
        }
    }
}



