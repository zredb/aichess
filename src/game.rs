use crate::ChessPlayer;
use crate::moves::Move;
use crate::position::Position;

impl mocats::Player for ChessPlayer {}

impl mocats::GameState<Move, ChessPlayer> for Position {
    fn get_actions(&self) -> Vec<Move> {
        todo!()
    }

    fn apply_action(&mut self, action: &Move) {
        todo!()
    }

    fn get_turn(&self) -> ChessPlayer {
        todo!()
    }

    fn get_reward_for_player(&self, player: ChessPlayer) -> f32 {
        todo!()
    }
}