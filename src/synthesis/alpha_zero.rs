use std::path::{Path, PathBuf};

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::distr::{weighted::WeightedIndex, Distribution};
use rand::prelude::*;
use rand::Rng;

use crate::synthesis::data::{FlatBatch, ReplayBuffer};
use crate::synthesis::game::Outcome;
use crate::synthesis::mcts::MCTS;
use crate::synthesis::utils::{git_diff, git_hash, save_str};
use crate::synthesis::{Game, LearningConfig, Policy, PolicyWithCache, RolloutConfig, ValueTarget};

#[derive(Debug, Clone, Default)]
pub struct TrainingMetrics {
    pub positions: usize,
    pub batches: usize,
    pub policy_loss: Option<f32>,
    pub value_loss: Option<f32>,
    pub total_loss: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct AlphaZeroIterationMetrics {
    pub iteration: usize,
    pub games_played: usize,
    pub fresh_steps: usize,
    pub replay_games: usize,
    pub replay_steps: usize,
    pub deduplicated_steps: usize,
    pub training: TrainingMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct AlphaZeroReport {
    pub iterations: Vec<AlphaZeroIterationMetrics>,
}

pub trait AlphaZeroTrainer<G: Game<N>, const N: usize> {
    type Policy: Policy<G, N>;

    fn save_checkpoint(&mut self, path: &Path) -> Result<()>;
    fn load_policy(&self, path: &Path) -> Result<Self::Policy>;
    fn train(
        &mut self,
        batch: &FlatBatch<G, N>,
        cfg: &LearningConfig,
        iteration: usize,
    ) -> Result<TrainingMetrics>;
}

pub fn alpha_zero<G, T, const N: usize>(
    cfg: &LearningConfig,
    trainer: &mut T,
) -> Result<AlphaZeroReport>
where
    G: 'static + Game<N>,
    T: AlphaZeroTrainer<G, N> + Sync,
{
    std::fs::create_dir_all(&cfg.logs)?;
    let models_dir = cfg.logs.join("models");
    std::fs::create_dir_all(&models_dir)?;
    save_str(&cfg.logs, "env_name", G::NAME)?;
    save_str(&cfg.logs, "git_hash", &git_hash()?)?;
    save_str(&cfg.logs, "git_diff.patch", &git_diff()?)?;

    let mut buffer = ReplayBuffer::new(buffer_capacity::<G, N>(cfg.games_to_keep));
    let mut report = AlphaZeroReport::default();

    trainer.save_checkpoint(&checkpoint_path(&models_dir, 0))?;

    for iteration in 0..cfg.num_iterations {
        let checkpoint = checkpoint_path(&models_dir, iteration);
        let fresh_steps =
            gather_experience::<G, T, N>(cfg, trainer, &checkpoint, &mut buffer, iteration)?;
        let deduplicated = buffer.deduplicate();
        let training = trainer.train(&deduplicated, cfg, iteration)?;
        trainer.save_checkpoint(&checkpoint_path(&models_dir, iteration + 1))?;

        report.iterations.push(AlphaZeroIterationMetrics {
            iteration,
            games_played: cfg.games_per_train,
            fresh_steps,
            replay_games: buffer.curr_games(),
            replay_steps: buffer.curr_steps(),
            deduplicated_steps: deduplicated.vs.len(),
            training,
        });
    }

    Ok(report)
}

fn checkpoint_path(models_dir: &Path, iteration: usize) -> PathBuf {
    models_dir.join(format!("model_{iteration}.ot"))
}

fn buffer_capacity<G, const N: usize>(games_to_keep: usize) -> usize
where
    G: Game<N>,
{
    G::MAX_TURNS.max(1) * games_to_keep.max(1)
}

fn gather_experience<G, T, const N: usize>(
    cfg: &LearningConfig,
    trainer: &T,
    checkpoint: &Path,
    buffer: &mut ReplayBuffer<G, N>,
    seed: usize,
) -> Result<usize>
where
    G: 'static + Game<N>,
    T: AlphaZeroTrainer<G, N> + Sync,
{
    let total_games = cfg.games_per_train;
    let worker_count = cfg.rollout_cfg.num_workers + 1;
    let multi_bar = MultiProgress::new();
    let mut worker_buffers = Vec::with_capacity(worker_count);

    std::thread::scope(|scope| -> Result<()> {
        let mut games_left = total_games;
        let mut workers_left = worker_count;
        let mut handles = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let num_games = games_left / workers_left;
            games_left -= num_games;
            workers_left -= 1;

            let worker_bar = multi_bar.add(styled_progress_bar(num_games));
            let worker_seed = (seed * worker_count + worker_index) as u64;
            let worker_cfg = cfg.clone();
            let worker_checkpoint = checkpoint.to_path_buf();

            handles.push(scope.spawn(move || {
                run_n_games::<G, T, N>(
                    worker_cfg,
                    trainer,
                    &worker_checkpoint,
                    num_games,
                    worker_bar,
                    worker_seed,
                )
            }));
        }

        for handle in handles {
            worker_buffers.push(handle.join().expect("self-play worker panicked")?);
        }

        Ok(())
    })?;

    buffer.keep_last_n_games(cfg.games_to_keep.saturating_sub(total_games));
    let fresh_steps = worker_buffers.iter().map(ReplayBuffer::curr_steps).sum();
    for worker_buffer in worker_buffers.iter_mut() {
        buffer.extend(worker_buffer);
    }

    Ok(fresh_steps)
}

fn styled_progress_bar(n: usize) -> ProgressBar {
    let bar = ProgressBar::new(n as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40}] {pos}/{len} ({percent}%) | {eta} remaining | {elapsed_precise}")
            .unwrap()
            .progress_chars("|| "),
    );
    bar
}

fn run_n_games<G, T, const N: usize>(
    cfg: LearningConfig,
    trainer: &T,
    checkpoint: &Path,
    num_games: usize,
    progress_bar: ProgressBar,
    seed: u64,
) -> Result<ReplayBuffer<G, N>>
where
    G: Game<N>,
    T: AlphaZeroTrainer<G, N>,
{
    let mut buffer = ReplayBuffer::new(G::MAX_TURNS.max(1) * num_games.max(1));
    let mut rng = StdRng::seed_from_u64(seed);
    let mut policy = trainer.load_policy(checkpoint)?;
    let mut cached_policy =
        PolicyWithCache::with_capacity(G::MAX_TURNS.max(1) * num_games.max(1), &mut policy);

    for _ in 0..num_games {
        buffer.new_game();
        run_game::<G, _, _, N>(&cfg.rollout_cfg, &mut cached_policy, &mut rng, &mut buffer);
        progress_bar.inc(1);
    }
    progress_bar.finish();

    Ok(buffer)
}

#[derive(Debug, Clone)]
struct StateInfo {
    turn: usize,
    t: f32,
    q: [f32; 3],
    z: [f32; 3],
}

impl StateInfo {
    fn from_q(turn: usize, q: [f32; 3]) -> Self {
        Self {
            turn,
            t: 0.0,
            q,
            z: [0.0; 3],
        }
    }
}

fn run_game<G, P, R, const N: usize>(
    cfg: &RolloutConfig,
    policy: &mut P,
    rng: &mut R,
    buffer: &mut ReplayBuffer<G, N>,
) where
    G: Game<N>,
    P: Policy<G, N>,
    R: Rng,
{
    let mut game = G::new();
    let mut solution = None;
    let mut search_policy = [0.0; N];
    let mut num_turns = 0;
    let mut state_infos = Vec::with_capacity(G::MAX_TURNS.max(1));

    while solution.is_none() {
        let mut mcts =
            MCTS::with_capacity(cfg.num_explores + 1, cfg.mcts_cfg, policy, game.clone());
        mcts.explore_n(cfg.num_explores);

        mcts.target_policy(&mut search_policy);
        buffer.add(&game, &search_policy, [0.0; 3]);
        state_infos.push(StateInfo::from_q(num_turns + 1, mcts.target_q()));

        let action = sample_action(cfg, &mut mcts, &game, &search_policy, rng, num_turns);
        solution = mcts.solution(&action);

        let is_over = game.step(&action);
        if is_over {
            solution = Some(game.reward(game.player()).into());
        } else if !cfg.stop_games_when_solved {
            solution = None;
        }
        num_turns += 1;
    }

    fill_state_info(
        &mut state_infos,
        solution.expect("game should finish").reversed(),
    );
    store_rewards(cfg, buffer, &state_infos);
}

fn sample_action<G, P, R, const N: usize>(
    cfg: &RolloutConfig,
    mcts: &mut MCTS<G, P, N>,
    game: &G,
    search_policy: &[f32; N],
    rng: &mut R,
    num_turns: usize,
) -> G::Action
where
    G: Game<N>,
    P: Policy<G, N>,
    R: Rng,
{
    let best = mcts.best_action(cfg.action);
    let solution = mcts.solution(&best);

    if num_turns < cfg.random_actions_until {
        let count = game.iter_actions().count();
        let n = rng.random_range(0..count);
        return game.iter_actions().nth(n).expect("legal action");
    }

    if num_turns < cfg.sample_actions_until && (solution.is_none() || !cfg.stop_games_when_solved) {
        if let Ok(dist) = WeightedIndex::new(search_policy.iter().copied()) {
            let choice = dist.sample(rng);
            return G::Action::from(choice);
        }
    }

    best
}

fn fill_state_info(state_infos: &mut [StateInfo], mut outcome: Outcome) {
    let num_turns = state_infos.len().max(1);
    for state_value in state_infos.iter_mut().rev() {
        state_value.z[match outcome {
            Outcome::Win(_) => 2,
            Outcome::Draw(_) => 1,
            Outcome::Lose(_) => 0,
        }] = 1.0;
        state_value.t = state_value.turn as f32 / num_turns as f32;
        outcome = outcome.reversed();
    }
}

fn store_rewards<G, const N: usize>(
    cfg: &RolloutConfig,
    buffer: &mut ReplayBuffer<G, N>,
    state_infos: &[StateInfo],
) where
    G: Game<N>,
{
    let num_turns = state_infos.len();
    let start_i = buffer.curr_steps().saturating_sub(num_turns);
    let end_i = buffer.curr_steps();
    for (buffer_value, state) in buffer.vs[start_i..end_i].iter_mut().zip(state_infos.iter()) {
        *buffer_value = match cfg.value_target {
            ValueTarget::Q => state.q,
            ValueTarget::Z => state.z,
            ValueTarget::QZaverage { p } => {
                let mut value = [0.0; 3];
                for (i, slot) in value.iter_mut().enumerate() {
                    *slot = state.q[i] * p + state.z[i] * (1.0 - p);
                }
                value
            }
            ValueTarget::QtoZ { from, to } => {
                let p = (1.0 - state.t) * from + state.t * to;
                let mut value = [0.0; 3];
                for (i, slot) in value.iter_mut().enumerate() {
                    *slot = state.q[i] * (1.0 - p) + state.z[i] * p;
                }
                value
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;

    use super::*;
    use crate::synthesis::{
        ActionSelection, Exploration, Fpu, HasTurnOrder, MCTSConfig, PolicyNoise,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Player {
        First,
        Second,
    }

    impl HasTurnOrder for Player {
        fn prev(&self) -> Self {
            self.next()
        }

        fn next(&self) -> Self {
            match self {
                Player::First => Player::Second,
                Player::Second => Player::First,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Action(usize);

    impl From<usize> for Action {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }

    impl From<Action> for usize {
        fn from(value: Action) -> Self {
            value.0
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct TinyGame {
        player: Player,
        turn: usize,
    }

    impl Game<2> for TinyGame {
        type PlayerId = Player;
        type Action = Action;
        type ActionIterator = std::vec::IntoIter<Action>;
        type Features = [f32; 2];

        const MAX_TURNS: usize = 2;
        const NAME: &'static str = "TinyGame";
        const NUM_PLAYERS: usize = 2;
        const DIMS: &'static [i64] = &[2];

        fn new() -> Self {
            Self {
                player: Player::First,
                turn: 0,
            }
        }

        fn player(&self) -> Self::PlayerId {
            self.player
        }

        fn is_over(&self) -> bool {
            self.turn >= Self::MAX_TURNS
        }

        fn reward(&self, player_id: Self::PlayerId) -> f32 {
            if self.turn < Self::MAX_TURNS {
                0.0
            } else if player_id == Player::First {
                1.0
            } else {
                -1.0
            }
        }

        fn iter_actions(&self) -> Self::ActionIterator {
            vec![Action(0), Action(1)].into_iter()
        }

        fn step(&mut self, _action: &Self::Action) -> bool {
            self.turn += 1;
            self.player = self.player.next();
            self.is_over()
        }

        fn features(&self) -> Self::Features {
            [
                self.turn as f32,
                if self.player == Player::First {
                    1.0
                } else {
                    -1.0
                },
            ]
        }

        fn print(&self) {}
    }

    #[derive(Clone, Default)]
    struct UniformPolicy;

    impl Policy<TinyGame, 2> for UniformPolicy {
        fn eval(&mut self, _game: &TinyGame) -> ([f32; 2], [f32; 3]) {
            ([0.5, 0.5], [0.0, 0.2, 0.8])
        }
    }

    #[derive(Default)]
    struct StubTrainer {
        trained_positions: Mutex<Vec<usize>>,
    }

    impl AlphaZeroTrainer<TinyGame, 2> for StubTrainer {
        type Policy = UniformPolicy;

        fn save_checkpoint(&mut self, path: &Path) -> Result<()> {
            std::fs::write(path, b"stub-checkpoint")?;
            Ok(())
        }

        fn load_policy(&self, path: &Path) -> Result<Self::Policy> {
            assert!(path.exists());
            Ok(UniformPolicy)
        }

        fn train(
            &mut self,
            batch: &FlatBatch<TinyGame, 2>,
            _cfg: &LearningConfig,
            _iteration: usize,
        ) -> Result<TrainingMetrics> {
            self.trained_positions
                .lock()
                .unwrap()
                .push(batch.states.len());
            Ok(TrainingMetrics {
                positions: batch.states.len(),
                batches: 1,
                policy_loss: Some(0.0),
                value_loss: Some(0.0),
                total_loss: Some(0.0),
            })
        }
    }

    #[test]
    fn alpha_zero_generates_self_play_and_saves_next_checkpoint() {
        let root = std::env::temp_dir().join(format!(
            "aichess-alphazero-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut trainer = StubTrainer::default();
        let cfg = LearningConfig {
            seed: 7,
            logs: root.clone(),
            lr_schedule: vec![(0, 1e-3)],
            weight_decay: 0.0,
            num_iterations: 1,
            num_epochs: 1,
            batch_size: 4,
            policy_weight: 1.0,
            value_weight: 1.0,
            games_to_keep: 8,
            games_per_train: 4,
            rollout_cfg: RolloutConfig {
                num_workers: 0,
                num_explores: 2,
                random_actions_until: 0,
                sample_actions_until: 1,
                stop_games_when_solved: true,
                value_target: ValueTarget::Z,
                action: ActionSelection::NumVisits,
                mcts_cfg: MCTSConfig {
                    exploration: Exploration::PolynomialUct { c: 1.25 },
                    solve: false,
                    correct_values_on_solve: false,
                    select_solved_nodes: false,
                    auto_extend: false,
                    fpu: Fpu::Const(0.0),
                    root_policy_noise: PolicyNoise::Equal { weight: 0.25 },
                },
            },
        };

        let report = alpha_zero::<TinyGame, _, 2>(&cfg, &mut trainer).unwrap();

        assert_eq!(report.iterations.len(), 1);
        assert!(report.iterations[0].fresh_steps > 0);
        assert!(root.join("models").join("model_1.ot").exists());
        assert!(!trainer.trained_positions.lock().unwrap().is_empty());

        let _ = std::fs::remove_dir_all(root);
    }
}
