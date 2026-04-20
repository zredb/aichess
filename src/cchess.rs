use crate::fen::{fen2_coords, Fen};
use crate::position::Position;
use crate::{Game, HasTurnOrder};
use crate::pos::moves::Move;

pub const BOARD_RANKS: usize = 10;
pub const BOARD_FILES: usize = 9;
pub const INPUT_PLANES: usize = 14;
pub const MAX_NUM_ACTIONS: usize = BOARD_RANKS * BOARD_FILES * BOARD_RANKS * BOARD_FILES;
const MAX_GAME_TURNS: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlayerId {
    Red,
    Black,
}

impl HasTurnOrder for PlayerId {
    fn prev(&self) -> Self {
        self.next()
    }

    fn next(&self) -> Self {
        match self {
            PlayerId::Black => PlayerId::Red,
            PlayerId::Red => PlayerId::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CChess {
    state: Fen,
    player: PlayerId,
}

impl CChess {
    fn winner(&self) -> Option<PlayerId> {
        let mut has_red_king = false;
        let mut has_black_king = false;
        for (piece, _) in fen2_coords(self.state.fen_str()) {
            match piece {
                'K' => has_red_king = true,
                'k' => has_black_king = true,
                _ => {}
            }
        }

        match (has_red_king, has_black_king) {
            (true, false) => Some(PlayerId::Red),
            (false, true) => Some(PlayerId::Black),
            (false, false) => None,
            (true, true) => {
                let position = Position::from_fen(&self.state);
                if position.gen_legal_moves().is_empty() {
                    Some(self.player.prev())
                } else {
                    None
                }
            }
        }
    }

    /// 获取当前状态的引用
    pub fn state(&self) -> &Fen {
        &self.state
    }
}

impl Game<MAX_NUM_ACTIONS> for CChess {
    type PlayerId = PlayerId;
    type Action = Move;
    type ActionIterator = MoveIterator;
    type Features = [[[f32; BOARD_FILES]; BOARD_RANKS]; INPUT_PLANES];

    const MAX_TURNS: usize = MAX_GAME_TURNS;
    const NAME: &'static str = "Chinese chess";
    const NUM_PLAYERS: usize = 2;
    const DIMS: &'static [i64] = &[INPUT_PLANES as i64, BOARD_RANKS as i64, BOARD_FILES as i64];

    fn new() -> Self {
        CChess {
            state: Fen::init(),
            player: PlayerId::Red,
        }
    }

    fn player(&self) -> Self::PlayerId { self.player }

    fn is_over(&self) -> bool {
        self.winner().is_some()
    }

    fn reward(&self, player_id: Self::PlayerId) -> f32
    {
        match self.winner() {
            Some(winner) => {
                if winner == player_id {
                    1.0
                } else {
                    -1.0
                }
            }
            None => 0.0,
        }
    }

    fn iter_actions(&self) -> Self::ActionIterator {
        let position = Position::from_fen(&self.state);
        MoveIterator {
            moves: position.gen_legal_moves(),
            index: 0,
        }
    }

    fn step(&mut self, action: &Self::Action) -> bool {
        let mut position = Position::from_fen(&self.state);
        position.make_move(*action);
        self.state = position.to_fen();
        self.player = self.player.next();
        self.is_over()
    }

    fn features(&self) -> Self::Features {
        // 使用 Copy trait，直接赋值而非 clone
        let mut features = [[[0.0; BOARD_FILES]; BOARD_RANKS]; INPUT_PLANES];

        for (piece, square) in fen2_coords(self.state.fen_str()) {
            let file = (square as usize & 0x0f).saturating_sub(3);
            let rank = ((square as usize) >> 4).saturating_sub(3);
            if rank >= BOARD_RANKS || file >= BOARD_FILES {
                continue;
            }

            let is_red_piece = piece.is_ascii_uppercase();
            let plane = match piece.to_ascii_uppercase() {
                'K' => 0,
                'A' => 1,
                'B' => 2,
                'N' => 3,
                'R' => 4,
                'C' => 5,
                'P' => 6,
                _ => continue,
            };

            let is_current_player_piece = match self.player {
                PlayerId::Red => is_red_piece,
                PlayerId::Black => !is_red_piece,
            };

            let plane = if is_current_player_piece { plane } else { plane + 7 };
            let (rank, file) = match self.player {
                PlayerId::Red => (rank, file),
                PlayerId::Black => (BOARD_RANKS - 1 - rank, BOARD_FILES - 1 - file),
            };
            features[plane][rank][file] = 1.0;
        }

        features
    }

    fn print(&self) {
        println!("{}", self.state);
    }
}

pub struct MoveIterator {
    moves: Vec<Move>,
    index: usize,
}

impl Iterator for MoveIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.moves.len() {
            let item = self.moves[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CChess, Game, INPUT_PLANES, BOARD_FILES, BOARD_RANKS, MAX_NUM_ACTIONS};

    #[test]
    fn cchess_initial_features_match_expected_tensor_shape() {
        let game = CChess::new();
        let features = game.features();

        assert_eq!(features.len(), INPUT_PLANES);
        assert_eq!(features[0].len(), BOARD_RANKS);
        assert_eq!(features[0][0].len(), BOARD_FILES);
        assert_eq!(CChess::MAX_NUM_ACTIONS, MAX_NUM_ACTIONS);
    }
}
