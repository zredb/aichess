
use std::collections::HashMap;
use crate::synthesis::game::Game;
use crate::synthesis::policies::Policy;

pub struct PolicyWithCache<'a, G: Game<N>, P: Policy<G, N>, const N: usize> {
    pub policy: &'a mut P,
    pub cache: HashMap<G, ([f32; N], [f32; 3])>,
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> PolicyWithCache<'a, G, P, N> {
    pub fn with_capacity(capacity: usize, policy: &'a mut P) -> Self {
        Self {
            policy,
            cache: HashMap::with_capacity(capacity),
        }
    }
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> Policy<G, N>
    for PolicyWithCache<'a, G, P, N>
{
    fn eval(&mut self, game: &G) -> ([f32; N], [f32; 3]) {
        match self.cache.get(game) {
            Some(pi_v) => *pi_v,
            None => {
                let pi_v = self.policy.eval(game);
                self.cache.insert(game.clone(), pi_v);
                pi_v
            }
        }
    }

    fn eval_batch(&mut self, games: &[G]) -> Vec<([f32; N], [f32; 3])> {
        let mut out = vec![([0.0; N], [0.0; 3]); games.len()];
        let mut miss_indices = Vec::new();
        let mut miss_games = Vec::new();

        for (i, game) in games.iter().enumerate() {
            if let Some(pi_v) = self.cache.get(game) {
                out[i] = *pi_v;
            } else {
                miss_indices.push(i);
                miss_games.push(game.clone());
            }
        }

        if !miss_games.is_empty() {
            let miss_results = self.policy.eval_batch(&miss_games);
            for ((i, game), pi_v) in miss_indices
                .into_iter()
                .zip(miss_games.into_iter())
                .zip(miss_results.into_iter())
            {
                self.cache.insert(game, pi_v);
                out[i] = pi_v;
            }
        }

        out
    }
}

pub struct OwnedPolicyWithCache<G: Game<N>, P: Policy<G, N>, const N: usize> {
    pub policy: P,
    pub cache: HashMap<G, ([f32; N], [f32; 3])>,
}

impl<G: Game<N>, P: Policy<G, N>, const N: usize> OwnedPolicyWithCache<G, P, N> {
    pub fn with_capacity(capacity: usize, policy: P) -> Self {
        Self {
            policy,
            cache: HashMap::with_capacity(capacity),
        }
    }
}

impl<G: Game<N>, P: Policy<G, N>, const N: usize> Policy<G, N> for OwnedPolicyWithCache<G, P, N> {
    fn eval(&mut self, game: &G) -> ([f32; N], [f32; 3]) {
        match self.cache.get(game) {
            Some(pi_v) => *pi_v,
            None => {
                let pi_v = self.policy.eval(game);
                self.cache.insert(game.clone(), pi_v);
                pi_v
            }
        }
    }

    fn eval_batch(&mut self, games: &[G]) -> Vec<([f32; N], [f32; 3])> {
        let mut out = vec![([0.0; N], [0.0; 3]); games.len()];
        let mut miss_indices = Vec::new();
        let mut miss_games = Vec::new();

        for (i, game) in games.iter().enumerate() {
            if let Some(pi_v) = self.cache.get(game) {
                out[i] = *pi_v;
            } else {
                miss_indices.push(i);
                miss_games.push(game.clone());
            }
        }

        if !miss_games.is_empty() {
            let miss_results = self.policy.eval_batch(&miss_games);
            for ((i, game), pi_v) in miss_indices
                .into_iter()
                .zip(miss_games.into_iter())
                .zip(miss_results.into_iter())
            {
                self.cache.insert(game, pi_v);
                out[i] = pi_v;
            }
        }

        out
    }
}
