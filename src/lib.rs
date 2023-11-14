mod game;
mod piece;
mod board;
mod moves;
mod fen;

// 横向
const WIDTH: usize = 9;
// 纵向
const HEIGHT: usize = 10;
// 最大位置数
const MAX_CELLS_SIZE: usize = 256;
// 最大棋子数
const MAX_PIECES_SIZE: usize = 32;

const STEP_INCREASE: bool = true;
const STEP_DECREASE: bool = false;
const PROCESS_ROW: bool = true;
const PROCESS_COLUMN: bool = false;

// 帅 士 相 马 车 炮 兵
const RED_KING: char = 'K';
const RED_ADVISER: char = 'A';
const RED_BISHOP: char = 'B';
const RED_KNIGHT: char = 'N';
const RED_ROOK: char = 'R';
const RED_CANNON: char = 'C';
const RED_PAWN: char = 'P';
const BLACK_KING: char = 'k';
const BLACK_ADVISER: char = 'a';
const BLACK_BISHOP: char = 'b';
const BLACK_KNIGHT: char = 'n';
const BLACK_ROOK: char = 'r';
const BLACK_CANNON: char = 'c';
const BLACK_PAWN: char = 'p';

