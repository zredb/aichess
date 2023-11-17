use rand::random;
use toto::{Toi32, Tou8};

use crate::position::{away_half, bishop_pin, file_disp, FILE_LEFT, file_x, in_board, in_fort, knight_pin, rank_disp, RANK_TOP, rank_y, same_half, square_forward};

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct SlideMove {
    pub(crate) ucNonCap: [u8; 2],
    pub(crate) ucRookCap: [u8; 2],
    pub(crate) ucCannonCap: [u8; 2],
    pub(crate) ucSuperCap: [u8; 2],
}

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct SlideMask {
    wNonCap: u16,
    wRookCap: u16,
    wCannonCap: u16,
    wSuperCap: u16,
}

#[derive(Default, Clone, Copy)]
struct Zobrist {
    dwKey: u32,
    dwLock0: u32,
    dwLock1: u32,
}

const CN_KING_MOVE_TAB: [i8; 4] = [-0x10, -0x01, 0x01, 0x10];
const CN_ADVISOR_MOVE_TAB: [i8; 4] = [-0x11, -0x0f, 0x0f, 0x11];
const CN_BISHOP_MOVE_TAB: [i8; 4] = [-0x22, -0x1e, 0x1e, 0x22];
const CN_KNIGHT_MOVE_TAB: [i8; 8] = [-0x21, -0x1f, -0x12, -0x0e, 0x0e, 0x12, 0x1f, 0x21];

impl Zobrist {
    fn init_rc4() -> Self {
        Zobrist {
            dwKey: random(),
            dwLock0: random(),
            dwLock1: random(),
        }
    }

    fn xor(&mut self, zobr: &Zobrist) {
        self.dwKey ^= zobr.dwKey;
        self.dwLock0 ^= zobr.dwLock0;
        self.dwLock1 ^= zobr.dwLock1;
    }

    fn xor2(&mut self, zobr1: &Zobrist, zobr2: &Zobrist) {
        self.dwKey ^= zobr1.dwKey ^ zobr2.dwKey;
        self.dwLock0 ^= zobr1.dwLock0 ^ zobr2.dwLock0;
        self.dwLock1 ^= zobr1.dwLock1 ^ zobr2.dwLock1;
    }
}

pub(crate) struct PreGen {
    zobrPlayer: Zobrist,
    zobrTable: [[Zobrist; 14]; 256],
    wBitRankMask: [u16; 256],
    wBitFileMask: [u16; 256],
    pub(crate) smvRankMoveTab: [[SlideMove; 512]; 9],
    pub(crate) smvFileMoveTab: [[SlideMove; 1024]; 10],
    pub(crate) smsRankMaskTab: [[SlideMask; 512]; 9],
    pub(crate) smsFileMaskTab: [[SlideMask; 1024]; 10],
    pub(crate) ucsqKingMoves: [[u8; 8]; 256],
    pub(crate) ucsqAdvisorMoves: [[u8; 8]; 256],
    pub(crate) ucsqBishopMoves: [[u8; 8]; 256],
    pub(crate) ucsqBishopPins: [[u8; 4]; 256],
    pub(crate) ucsqKnightMoves: [[u8; 12]; 256],
    pub(crate) ucsqKnightPins: [[u8; 8]; 256],
    pub(crate) ucsqPawnMoves: [[[u8; 4]; 256]; 2],
}

impl PreGen {
    pub(crate) fn new() -> Self {
        let mut smv: SlideMove = SlideMove {
            ucNonCap: [0; 2],
            ucRookCap: [0; 2],
            ucCannonCap: [0; 2],
            ucSuperCap: [0; 2],
        };
        let mut sms: SlideMask = SlideMask {
            wNonCap: 0,
            wRookCap: 0,
            wCannonCap: 0,
            wSuperCap: 0,
        };

        // 首先初始化Zobrist键值表
        let mut zobrTable: [[Zobrist; 14]; 256] = [[Zobrist::default(); 14]; 256];

        for i in 0..256 {
            for j in 0..14 {
                zobrTable[i][j] = Zobrist::init_rc4();
            }
        }

        let mut wBitRankMask: [u16; 256] = [0; 256];
        let mut wBitFileMask: [u16; 256] = [0; 256];
        // 然后初始化屏蔽位行和屏蔽位列
        // 注：位行和位列不包括棋盘以外的位，所以就会频繁使用"+/- RANK_TOP/FILE_LEFT"
        for sq_src in 0..256usize {
            if in_board(sq_src as i32) {
                wBitRankMask[sq_src] = 1 << (file_x(sq_src) - FILE_LEFT);
                wBitFileMask[sq_src] = 1 << (rank_y(sq_src) - RANK_TOP);
            }
        }
        let mut smv = SlideMove::default();
        let mut sms = SlideMask::default();
        let mut smvRankMoveTab = [[smv; 512]; 9];
        let mut smvFileMoveTab = [[smv; 1024]; 10];
        let mut smsRankMaskTab = [[sms; 512]; 9];
        let mut smsFileMaskTab = [[sms; 1024]; 10];
        // 然后生成车炮横向的预置数组
        for i in 0..9 {
            for j in 0..512 { //车:256;炮:256和为512
                // 初始化借助于“位行”的车和炮的着法预生成数组，包括以下几个步骤：
                // 1. 初始化临时变量"SlideMoveTab"，假设没有着法，用起始格填充
                let p = (i + FILE_LEFT) as u8;
                smv.ucNonCap[0] = p;
                smv.ucNonCap[1] = p;
                smv.ucRookCap[0] = p;
                smv.ucRookCap[1] = p;
                smv.ucCannonCap[0] = p;
                smv.ucCannonCap[1] = p;
                smv.ucSuperCap[0] = p;
                smv.ucSuperCap[1] = p;
                sms.wNonCap = 0;
                sms.wRookCap = 0;
                sms.wCannonCap = 0;
                sms.wSuperCap = 0;
                // 提示：参阅"pregen.h"，...[0]表示最大一格，向右移动和下移动都用[0]，反之亦然
                // 2. 考虑向右移动的目标格，填充...[0]，
                let mut outerk = 0usize;
                for k in (i + 1)..=8 {
                    if j & (1 << k) != 0 {
                        smv.ucRookCap[0] = file_disp(k + FILE_LEFT);
                        sms.wRookCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                    smv.ucNonCap[0] = file_disp(k + FILE_LEFT);
                    sms.wNonCap |= 1 << k;
                }
                for k in (outerk + 1)..=8 {
                    if j & (1 << k) != 0 {
                        smv.ucCannonCap[0] = file_disp(k + FILE_LEFT);
                        sms.wCannonCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                for k in (outerk + 1)..=8 {
                    if j & (1 << k) != 0 {
                        smv.ucSuperCap[0] = file_disp(k + FILE_LEFT);
                        sms.wSuperCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                // 3. 考虑向左移动的目标格，填充...[1]
                for k in i..=0 {
                    if j & (1 << k) != 0 {
                        smv.ucRookCap[1] = file_disp(k + FILE_LEFT);
                        sms.wRookCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                    smv.ucNonCap[1] = file_disp(k + FILE_LEFT);
                    sms.wNonCap |= 1 << k;
                }
                let c = if outerk > 1 { outerk - 1 } else { 0 };
                for k in c..=0 {
                    if j & (1 << k) != 0 {
                        smv.ucCannonCap[1] = file_disp(k + FILE_LEFT);
                        sms.wCannonCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                let c = if outerk > 1 { outerk - 1 } else { 0 };
                for k in c..=0 {
                    if j & (1 << k) != 0 {
                        smv.ucSuperCap[1] = file_disp(k + FILE_LEFT);
                        sms.wSuperCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                smvRankMoveTab[i][j] = smv;
                smsRankMaskTab[i][j] = sms;
            }
        }
        // 然后生成车炮横向的预置数组(数组的应用参阅"pregen.h")
        for i in 0..10 {
            for j in 0..1024 {
                let p = ((i + RANK_TOP) * 16) as u8;
                smv.ucNonCap[0] = p;
                smv.ucNonCap[1] = p;
                smv.ucRookCap[0] = p;
                smv.ucRookCap[1] = p;
                smv.ucCannonCap[0] = p;
                smv.ucCannonCap[1] = p;
                smv.ucSuperCap[0] = p;
                smv.ucSuperCap[1] = p;
                sms.wNonCap = 0;
                sms.wRookCap = 0;
                sms.wCannonCap = 0;
                sms.wSuperCap = 0;
                let mut outerk = 0usize;
                for k in (i + 1)..=9 {
                    if j & (1 << k) != 0 {
                        smv.ucRookCap[0] = rank_disp(k + RANK_TOP);
                        sms.wRookCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                    smv.ucNonCap[0] = rank_disp(k + RANK_TOP);
                    sms.wNonCap |= 1 << k;
                }
                for k in (outerk + 1)..=9 {
                    if j & (1 << k) != 0 {
                        smv.ucCannonCap[0] = rank_disp(k + RANK_TOP);
                        sms.wCannonCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                for k in (outerk + 1)..=9 {
                    if j & (1 << k) != 0 {
                        smv.ucSuperCap[0] = rank_disp(k + RANK_TOP);
                        sms.wSuperCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                let c = if i > 1 { i - 1 } else { 0 };
                for k in c..=0 {
                    if j & (1 << k) != 0 {
                        smv.ucRookCap[1] = rank_disp(k + RANK_TOP);
                        sms.wRookCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                    smv.ucNonCap[1] = rank_disp(k + RANK_TOP);
                    sms.wNonCap |= 1 << k;
                }
                let c = if outerk > 1 { outerk - 1 } else { 0 };
                for k in c..=0 {
                    if j & (1 << k) != 0 {
                        smv.ucCannonCap[1] = rank_disp(k + RANK_TOP);
                        sms.wCannonCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }
                let c = if outerk > 1 { outerk - 1 } else { 0 };
                for k in c..=0 {
                    if j & (1 << k) != 0 {
                        smv.ucSuperCap[1] = rank_disp(k + RANK_TOP);
                        sms.wSuperCap |= 1 << k;
                        outerk = k;
                        break;
                    }
                }

                smvFileMoveTab[i][j] = smv;
                smsFileMaskTab[i][j] = sms;
            }
        }
        let mut ucsqKingMoves = [[0u8; 8]; 256];
        let mut ucsqKingMoves: [[u8; 8]; 256] = [[0u8; 8]; 256];
        let mut ucsqAdvisorMoves: [[u8; 8]; 256] = [[0u8; 8]; 256];
        let mut ucsqBishopMoves: [[u8; 8]; 256] = [[0u8; 8]; 256];
        let mut ucsqBishopPins: [[u8; 4]; 256] = [[0u8; 4]; 256];
        let mut ucsqKnightMoves: [[u8; 12]; 256] = [[0u8; 12]; 256];
        let mut ucsqKnightPins: [[u8; 8]; 256] = [[0u8; 8]; 256];
        let mut ucsqPawnMoves: [[[u8; 4]; 256]; 2] = [[[0u8; 4]; 256]; 2];
        for sq_src in 0..256 {
            if in_board(sq_src) {
                // 生成帅(将)的着法预生成数组
                let mut n = 0;
                for i in 0..4 {
                    let sq_dst = sq_src + CN_KING_MOVE_TAB[i].to_i32();
                    if in_fort(sq_dst) {
                        ucsqKingMoves[sq_src as usize][n] = sq_dst.to_u8();
                        n += 1;
                    }
                }
                assert!(n <= 4);
                ucsqKingMoves[sq_src as usize][n] = 0;

                // 生成仕(士)的着法预生成数组
                n = 0;
                for i in 0..4 {
                    let sq_dst = sq_src as i32 + CN_ADVISOR_MOVE_TAB[i].to_i32();
                    if sq_dst >= 0 && in_fort(sq_dst) {
                        ucsqAdvisorMoves[sq_src as usize][n] = sq_dst.to_u8();
                        n += 1;
                    }
                }
                assert!(n <= 4);
                ucsqAdvisorMoves[sq_src as usize][n] = 0;

                // 生成相(象)的着法预生成数组，包括象眼数组
                n = 0;
                for i in 0..4 {
                    let sq_dst = sq_src + CN_BISHOP_MOVE_TAB[i].to_i32();
                    if in_board(sq_dst) && same_half(sq_src, sq_dst) {
                        ucsqBishopMoves[sq_src as usize][n] = sq_dst.to_u8();
                        ucsqBishopPins[sq_src as usize][n] = bishop_pin(sq_src, sq_dst);
                        n += 1;
                    }
                }
                assert!(n <= 4);
                ucsqBishopMoves[sq_src as usize][n] = 0;

                // 生成马的着法预生成数组，包括马腿数组
                n = 0;
                for i in 0..8 {
                    let sq_dst = sq_src + CN_KNIGHT_MOVE_TAB[i].to_i32();
                    if in_board(sq_dst) {
                        ucsqKnightMoves[sq_src as usize][n] = sq_dst.to_u8();
                        ucsqKnightPins[sq_src as usize][n] = knight_pin(sq_src, sq_dst);
                        n += 1;
                    }
                }
                assert!(n <= 8);
                ucsqKnightMoves[sq_src as usize][n] = 0;

                // 生成兵(卒)的着法预生成数组
                for i in 0..2 {
                    n = 0;
                    let mut sq_dst = square_forward(sq_src, i);
                    sq_dst = sq_src + if i == 0 { -16 } else { 16 };
                    if in_board(sq_dst) {
                        ucsqPawnMoves[i as usize][sq_src as usize][n] = sq_dst.to_u8();
                        n += 1;
                    }
                    if away_half(sq_src, i) { //过了河
                        for j in -1..=1 {
                            sq_dst = sq_src + j;
                            if in_board(sq_dst) {
                                ucsqPawnMoves[i as usize][sq_src as usize][n] = sq_dst.to_u8();
                                n += 1;
                            }
                        }
                    }
                }
            }
        }


        let pregen = PreGen {
            zobrPlayer: Zobrist::init_rc4(),
            zobrTable,
            wBitRankMask,
            wBitFileMask,
            smvRankMoveTab,
            smvFileMoveTab,
            smsRankMaskTab,
            smsFileMaskTab,
            ucsqKingMoves,
            ucsqAdvisorMoves,
            ucsqBishopMoves,
            ucsqBishopPins,
            ucsqKnightMoves,
            ucsqKnightPins,
            ucsqPawnMoves,
        };
        pregen
    }
}


