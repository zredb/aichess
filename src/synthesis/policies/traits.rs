use burn::prelude::{Backend, Tensor};
use crate::synthesis::game::Game;

pub trait Policy<G: Game<N>, const N: usize> {
    fn eval(&mut self, game: &G) -> ([f32; N], [f32; 3]);
}

pub trait NNPolicy<B:Backend,G: Game<N>, const N: usize> {
    //fn new(vs: &VarStore) -> Self;
    fn forward(&self, xs: &Tensor<B,2>) -> (&Tensor<B,2>, &Tensor<B,2>);
}
