use crate::fen::Fen;
use crate::{Game, HasTurnOrder};
use crate::pos::moves::Move;

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
    type ActionIterator = ();
    type Features = ();
    const MAX_TURNS: usize = 0;
    const NAME: &'static str = "Chinese chess";
    const NUM_PLAYERS: usize = 2;
    const DIMS: &'static [i64] = &[];

    fn new() -> Self {
        CChess {
            state: Fen::new(),
            player: PlayerId::Red,
        }
    }

    fn player(&self) -> Self::PlayerId {self.player}

    fn is_over(&self) -> bool {
        self.winner().is_some()
    }

    fn reward(&self, player_id: Self::PlayerId) -> f32 {
        todo!()
    }

    fn iter_actions(&self) -> Self::ActionIterator {
        todo!()
    }

    fn step(&mut self, action: &Self::Action) -> bool {
        todo!()
    }

    fn features(&self) -> Self::Features {
        todo!()
    }

    fn print(&self) {
        todo!()
    }
}

