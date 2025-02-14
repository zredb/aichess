use std::collections::HashSet;
use rand_distr::num_traits::ToPrimitive;
use toto::Toi32;

use crate::moves::Move;
use crate::fen::Fen;
use crate::pos::{ADVISOR_FROM, ADVISOR_TO, ADVISOR_TYPE, BISHOP_FROM, BISHOP_TO, BISHOP_TYPE, bit_piece, CANNON_FROM, CANNON_TO, CANNON_TYPE, ChessPlayer, coord_xy, file_disp, KING_FROM, KING_TYPE, KNIGHT_FROM, KNIGHT_TO, KNIGHT_TYPE, opp_side_tag2, PAWN_FROM, PAWN_TO, PAWN_TYPE, piece_char, piece_char_with_side, piece_type, rank_disp, ROOK_FROM, ROOK_TO, ROOK_TYPE, side_tag, side_tag2};
use crate::pos::pregen::{PreGen, SlideMask, SlideMove, Zobrist};
use crate::{FILE_LEFT, FILE_RIGHT, file_x, RANK_BOTTOM, RANK_TOP, rank_y};

///局面
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Position {
    // 轮到哪方走，0表示红方，1表示黑方
    current_player: ChessPlayer,
    // 每个格子放的棋子，None表示没有棋子
    ucpc_squares: [u8; 256],
    // 每个棋子放的位置，0表示被吃
    ucsq_pieces: [u8; 48],
    dw_bit_piece: u32,
    // 32位的棋子位，0到31位依次表示序号为16到47的棋子是否还在棋盘上
    w_bit_ranks: [u16; 16],
    // 位行数组，注意用法是"w_bit_ranks[RANK_Y(sq)]"
    w_bit_files: [u16; 16],
    // 位列数组，注意用法是"w_bit_files[FILE_X(sq)]"
    pre_gen: PreGen,
    zobr: Zobrist, // Zobrist
    winner: Option<ChessPlayer>,
}


impl Position {
    /// 新建一个棋盘对象。
    pub fn new() -> Position {
        Position {
            current_player: ChessPlayer::Red,
            ucpc_squares: [0; 256],
            ucsq_pieces: [0; 48],
            dw_bit_piece: 0,
            w_bit_ranks: [0; 16],
            w_bit_files: [0; 16],
            pre_gen: PreGen::new(),
            zobr: Zobrist::init_rc4(),
            winner: None,
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

    pub fn from_fen(fen: &Fen) -> Self {
        let sz_fen = fen.fen_str();
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
                position.change_side2();
            }
        }

        position
    }
    fn change_side2(&mut self) {
        match self.current_player {
            ChessPlayer::Red => self.current_player = ChessPlayer::Black,
            ChessPlayer::Black => self.current_player = ChessPlayer::Black,
        }
    }
    fn add_piece(&mut self, sq: usize, pc: usize) {
        self.ucpc_squares[sq] = pc as u8;
        self.ucsq_pieces[pc] = sq as u8;
        self.w_bit_files[file_x(sq)] ^= self.pre_gen.w_bit_file_mask[sq];
        self.w_bit_ranks[rank_y(sq)] ^= self.pre_gen.w_bit_rank_mask[sq];
        self.dw_bit_piece ^= bit_piece(pc);
        let mut ppt = piece_type(pc);
        if pc >= 32 {
            ppt += 7;
        }
        self.zobr.xor(&self.pre_gen.zobr_table[sq][ppt as usize]);
    }

    fn del_piece(&mut self, sq: usize, pc: usize) {
        self.ucpc_squares[sq] = 0;
        self.ucsq_pieces[pc] = 0;
        self.w_bit_files[file_x(sq)] ^= self.pre_gen.w_bit_file_mask[sq];
        self.w_bit_ranks[rank_y(sq)] ^= self.pre_gen.w_bit_rank_mask[sq];
        self.dw_bit_piece ^= bit_piece(pc);
        let mut ppt = piece_type(pc);
        if pc >= 32 {
            ppt += 7;
        }
        self.zobr.xor(&self.pre_gen.zobr_table[ppt as usize][sq]);
    }

    pub(crate) fn to_fen(&self) -> Fen {
        let mut fen = String::new();
        for i in RANK_TOP..=RANK_BOTTOM {
            let mut k = 0;
            for j in FILE_LEFT..=FILE_RIGHT {
                let pc = self.ucpc_squares[coord_xy(j, i)];
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

        if self.current_player == ChessPlayer::Red {
            fen.push('w');
        } else {
            fen.push('b');
        }

        Fen::new(&fen)
    }
    fn w_bit_piece(&self) -> u32 {
        match self.current_player {
            ChessPlayer::Red => self.dw_bit_piece & 0xffff,
            ChessPlayer::Black => self.dw_bit_piece >> 16,
        }
    }

    fn get_piece_char(&self, pt: usize) -> char {
        match self.current_player {
            ChessPlayer::Red => piece_char(pt),
            ChessPlayer::Black => piece_char(pt).to_ascii_lowercase(),
        }
    }

    fn rank_move(&self, x: usize, y: usize) -> &SlideMove {
        let adjusted_x = x - FILE_LEFT;
        let rank_move_tab = &self.pre_gen.smv_rank_move_tab[adjusted_x];
        let w_bit_ranks = &self.w_bit_ranks[y];
        let res = &rank_move_tab[*w_bit_ranks as usize];
        res
    }

    fn file_move(&self, x: usize, y: usize) -> &SlideMove {
        let adjusted_y = y - RANK_TOP;
        let file_move_tab = &self.pre_gen.smv_file_move_tab[adjusted_y];
        let w_bit_files = &self.w_bit_files[x];
        let res = &file_move_tab[*w_bit_files as usize];
        res
    }

    fn rank_mask(&self, x: usize, y: usize) -> &SlideMask {
        let adjusted_x = x - FILE_LEFT;
        let rank_move_tab = &self.pre_gen.sms_rank_mask_tab[adjusted_x];
        let w_bit_ranks = &self.w_bit_ranks[y];
        let res = &rank_move_tab[*w_bit_ranks as usize];
        res
    }

    fn file_mask(&self, x: usize, y: usize) -> &SlideMask {
        let adjusted_y = y - RANK_TOP;
        let file_mask_tab = &self.pre_gen.sms_file_mask_tab[adjusted_y];
        let w_bit_files = &self.w_bit_files[x];
        let res = &file_mask_tab[*w_bit_files as usize];
        res
    }

    pub(crate) fn gen_legal_moves(&self) -> Vec<Move> {
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
                let sq_dst = lpsmv.uc_rook_cap[0] + rank_disp(y);
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
                let sq_dst = lpsmv.uc_rook_cap[1] + rank_disp(y);
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
                let sq_dst = lpsmv.uc_rook_cap[0] + file_disp(x);
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
                let sq_dst = lpsmv.uc_rook_cap[1] + file_disp(x);
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
                let sq_dst = lpsmv.uc_cannon_cap[0] + rank_disp(y);
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
                let sq_dst = lpsmv.uc_cannon_cap[1] + rank_disp(y);
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
                let sq_dst = lpsmv.uc_cannon_cap[0] + file_disp(x);
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
                let sq_dst = lpsmv.uc_cannon_cap[1] + file_disp(x);

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
                    self.pre_gen.ucsq_pawn_moves[self.current_player as usize][sq_src as usize];
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
                let mut sq_dst = lpsmv.uc_non_cap[0] + rank_disp(y);
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
                sq_dst = lpsmv.uc_non_cap[1] + rank_disp(y);
                while sq_dst != sq_src && sq_dst > 0 {
                    let mv = Move::new(self.get_piece_char(piece_type), sq_src as usize, sq_dst);
                    res.push(mv);
                    sq_dst += 1;
                }
                let lpsmv = self.file_move(x, y);
                sq_dst = lpsmv.uc_non_cap[0] + file_disp(x);
                while sq_dst != sq_src && sq_dst > 0 {
                    let mv = Move::new(self.get_piece_char(piece_type), sq_src as usize, sq_dst);
                    res.push(mv);
                    sq_dst -= 16;
                }
                sq_dst = lpsmv.uc_non_cap[1] + file_disp(x);
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
                    self.pre_gen.ucsq_pawn_moves[self.current_player as usize][sq_src as usize];
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

    pub fn make_move(&mut self, mv: Move) {
        let pc_dst = self.ucpc_squares[mv.to as usize];
        if (pc_dst > 0) { //目标位置有棋子, 先去掉目标位置的棋子,
            self.del_piece(mv.to as usize, pc_dst as usize);
            let pt=piece_char(pc_dst as usize);
            if pt == 'K' {
               self.winner=Some(self.current_player);
            }
        }
        // 把源位置上的棋子移动到目标位置
        let pc = self.ucpc_squares[mv.from];
        self.add_piece(mv.to as usize, pc as usize); //添加目标位置棋子
        self.del_piece(mv.from as usize, pc as usize); //移除源位置棋子
    }
    fn check_mate(&self) -> bool {
        todo!()
    }
}

pub fn fen_piece(n_arg: char) -> usize {
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

    use crate::position::{rank_y, Position};
    use crate::{away_half, square_forward};
    use crate::fen::Fen;

    #[test]
    fn test_fen() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let fen1 = Fen::new(fen);
        let board = Position::from_fen(&fen1);
        let fen2 = Position::to_fen(&board);
        assert_eq!(fen1, fen2);
        //let moves = board.generate_all_moves(&Side::Red);
    }

    #[test]
    fn test_gen_cap_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(&Fen::new(fen));
        let moves = position.gen_cap_moves();
        assert_eq!(moves.len(), 2);
    }

    #[test]
    fn test_gen_nocap_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(&Fen::new(fen));
        let moves = position.gen_nocap_moves();
        for mv in moves.iter().filter(|mv| mv.piece == 'C' || mv.piece == 'R') {
            println!("{}", &mv.to_string());
        }

        assert_eq!(moves.len(), 42);
    }
    #[test]
    fn test_gen_moves() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(&Fen::new(fen));
        let moves = position.gen_legal_moves();
        assert_eq!(moves.len(), 44);
    }
    #[test]
    fn test_piece_loc() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let position = Position::from_fen(&Fen::new(fen));
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
