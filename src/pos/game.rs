use crate::pos::ChessPlayer;
use crate::pos::moves::Move;

pub(crate) fn generate_inputs(state:&str, current_turn:ChessPlayer){
    //todo
}
pub(crate) fn is_black_turn(current_turn:ChessPlayer)->bool{
    current_turn == ChessPlayer::Black
}

pub(crate) fn flip_policy(){
    //todo    
}
pub(crate) fn get_legal_moves(state:&str, current_turn:ChessPlayer)->Vec<Move>{
    //todo
    todo!()
}
