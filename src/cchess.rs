use crate::fen::Fen;
use crate::{Game, HasTurnOrder, Policy};
use crate::pos::moves::Move;
use crate::position::Position;

const MAX_NUM_ACTIONS: usize = 2086;
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

const fn won(state: &Fen) -> bool {
   todo!()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CChess {
    state: Fen,
    player: PlayerId,
}

impl CChess {
    fn winner(&self) -> Option<PlayerId> {
        if won(&self.state) {
            Some(self.player.next())
        } else {
            None
        }
    }
}

impl Game<MAX_NUM_ACTIONS> for CChess {
    type PlayerId = PlayerId;
    type Action = Move;
    type ActionIterator =MoveIterator;
    type Features = ();
    const MAX_TURNS: usize = 0;
    const NAME: &'static str = "Chinese chess";
    const NUM_PLAYERS: usize = 2;
    const DIMS: &'static [i64] = &[];

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
        let mut position =  Position::from_fen(&self.state);
        position.make_move(*action);
        self.state= position.to_fen();
        self.player = self.player.next();
        self.winner().is_none()
    }

    fn features(&self) -> Self::Features {
        todo!()
    }

    fn print(&self) {
        println!("{}", self.state);
    }
}


impl Policy<CChess, { CChess::MAX_NUM_ACTIONS }> for CChess {
    fn eval(&mut self, env: &CChess) -> ([f32; CChess::MAX_NUM_ACTIONS], [f32; 3]) {
        let xs = env.features();
        let t = tensor(&xs, Connect4::DIMS, tch::Kind::Float);
        let (logits, value) = self.forward(&t);
        let mut policy = [0.0f32; Connect4::MAX_NUM_ACTIONS];
        logits.copy_data(&mut policy, Connect4::MAX_NUM_ACTIONS);
        let mut outcomes = [0.0f32; 3];
        value
            .softmax(-1, tch::Kind::Float)
            .copy_data(&mut outcomes, 3);
        (policy, outcomes)
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
            let item = self.moves[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}