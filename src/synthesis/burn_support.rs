use std::path::{Path, PathBuf};

use anyhow::Result;
use burn::backend::{Autodiff, Wgpu};
use burn::module::Module;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::record::{FullPrecisionSettings, NamedMpkFileRecorder};
use burn::tensor::activation::{log_softmax, softmax};
use burn::tensor::{Tensor, TensorData};

use crate::cchess::{CChess, BOARD_FILES, BOARD_RANKS, INPUT_PLANES, MAX_NUM_ACTIONS};
use crate::net::{Net, NetConfig};
use crate::synthesis::data::FlatBatch;
use crate::synthesis::{AlphaZeroTrainer, Game, LearningConfig, Policy, TrainingMetrics};

pub type BurnBackend = Wgpu<f32, i32>;
pub type BurnAutodiffBackend = Autodiff<BurnBackend>;

#[derive(Debug, Clone)]
pub struct BurnPolicy {
    model: Net<BurnBackend>,
    device: <BurnBackend as burn::prelude::Backend>::Device,
}

impl BurnPolicy {
    pub fn new(
        model: Net<BurnBackend>,
        device: <BurnBackend as burn::prelude::Backend>::Device,
    ) -> Self {
        Self { model, device }
    }
}

impl Policy<CChess, MAX_NUM_ACTIONS> for BurnPolicy {
    fn eval(&mut self, game: &CChess) -> ([f32; MAX_NUM_ACTIONS], [f32; 3]) {
        let input = state_tensor::<BurnBackend>(&[game.features()], &self.device);
        let (policy_logits, value_logits) = self.model.forward(input);
        let value_probs = softmax(value_logits, 1);

        let mut policy = [0.0; MAX_NUM_ACTIONS];
        let mut value = [0.0; 3];

        let policy_values = policy_logits.into_data().to_vec::<f32>().unwrap();
        let value_values = value_probs.into_data().to_vec::<f32>().unwrap();
        policy.copy_from_slice(&policy_values[..MAX_NUM_ACTIONS]);
        value.copy_from_slice(&value_values[..3]);

        (policy, value)
    }
}

#[derive(Debug, Clone)]
pub struct BurnTrainer {
    model: Net<BurnAutodiffBackend>,
    model_config: NetConfig,
    device: <BurnBackend as burn::prelude::Backend>::Device,
}

impl BurnTrainer {
    pub fn new(
        model_config: NetConfig,
        device: <BurnBackend as burn::prelude::Backend>::Device,
    ) -> Self {
        let model = model_config.init::<BurnAutodiffBackend>(&device);
        Self {
            model,
            model_config,
            device,
        }
    }

    fn checkpoint_base(path: &Path) -> PathBuf {
        let mut base = path.to_path_buf();
        if base.extension().is_some() {
            base.set_extension("");
        }
        base
    }

    fn learning_rate(cfg: &LearningConfig, iteration: usize) -> f64 {
        cfg.lr_schedule
            .iter()
            .filter(|(scheduled_iteration, _)| *scheduled_iteration <= iteration + 1)
            .last()
            .or_else(|| cfg.lr_schedule.first())
            .map(|(_, lr)| *lr)
            .unwrap_or(1e-3)
    }
}

impl Default for BurnTrainer {
    fn default() -> Self {
        Self::new(NetConfig::new(256, 7), Default::default())
    }
}

// SAFETY: BurnTrainer contains wgpu device which is thread-safe in practice,
// even though the type system doesn't reflect this
unsafe impl Sync for BurnTrainer {}
unsafe impl Send for BurnTrainer {}

impl AlphaZeroTrainer<CChess, MAX_NUM_ACTIONS> for BurnTrainer {
    type Policy = BurnPolicy;

    fn save_checkpoint(&mut self, path: &Path) -> Result<()> {
        let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
        let base = Self::checkpoint_base(path);
        self.model
            .clone()
            .save_file(base, &recorder)?;
        std::fs::write(path, b"burn checkpoint metadata")?;
        Ok(())
    }

    fn load_policy(&self, path: &Path) -> Result<Self::Policy> {
        let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
        let base = Self::checkpoint_base(path);
        let model = self
            .model_config
            .init::<BurnBackend>(&self.device)
            .load_file(base, &recorder, &self.device)?;
        Ok(BurnPolicy::new(model, self.device.clone()))
    }

    fn train(
        &mut self,
        batch: &FlatBatch<CChess, MAX_NUM_ACTIONS>,
        cfg: &LearningConfig,
        iteration: usize,
    ) -> Result<TrainingMetrics> {
        if batch.states.is_empty() {
            return Ok(TrainingMetrics::default());
        }

        let batch_size = (cfg.batch_size as usize).max(1);
        let lr = Self::learning_rate(cfg, iteration);
        let mut indices: Vec<usize> = (0..batch.states.len()).collect();
        let mut rng = StdRng::seed_from_u64(cfg.seed + iteration as u64);
        indices.shuffle(&mut rng);

        let mut optimizer = AdamConfig::new().init();
        let mut model = self.model.clone();
        let mut total_policy_loss = 0.0;
        let mut total_value_loss = 0.0;
        let mut total_loss = 0.0;
        let mut num_batches = 0usize;

        for _ in 0..cfg.num_epochs {
            indices.shuffle(&mut rng);
            for chunk in indices.chunks(batch_size) {
                let states: Vec<_> = chunk.iter().map(|&idx| batch.states[idx].clone()).collect();
                let pis: Vec<_> = chunk.iter().map(|&idx| batch.pis[idx]).collect();
                let values: Vec<_> = chunk.iter().map(|&idx| batch.vs[idx]).collect();

                let state_tensor = state_tensor::<BurnAutodiffBackend>(&states, &self.device);
                let pi_tensor = policy_tensor::<BurnAutodiffBackend>(&pis, &self.device);
                let value_tensor = value_tensor::<BurnAutodiffBackend>(&values, &self.device);

                let (policy_logits, value_logits) = model.forward(state_tensor);
                let policy_loss = cross_entropy(policy_logits, pi_tensor);
                let value_loss = cross_entropy(value_logits, value_tensor);
                let loss = policy_loss.clone().mul_scalar(cfg.policy_weight)
                    + value_loss.clone().mul_scalar(cfg.value_weight);

                let grads = loss.clone().backward();
                let grads = GradientsParams::from_grads(grads, &model);
                model = optimizer.step(lr, model, grads);

                total_policy_loss += scalar(&policy_loss);
                total_value_loss += scalar(&value_loss);
                total_loss += scalar(&loss);
                num_batches += 1;
            }
        }

        self.model = model;

        Ok(TrainingMetrics {
            positions: batch.states.len(),
            batches: num_batches,
            policy_loss: Some(total_policy_loss / num_batches.max(1) as f32),
            value_loss: Some(total_value_loss / num_batches.max(1) as f32),
            total_loss: Some(total_loss / num_batches.max(1) as f32),
        })
    }
}

fn state_tensor<B: burn::prelude::Backend>(
    states: &[<CChess as crate::synthesis::Game<MAX_NUM_ACTIONS>>::Features],
    device: &B::Device,
) -> Tensor<B, 4> {
    let mut data = Vec::with_capacity(states.len() * INPUT_PLANES * BOARD_RANKS * BOARD_FILES);
    for state in states {
        for plane in state {
            for row in plane {
                for value in row {
                    data.push(*value);
                }
            }
        }
    }

    Tensor::<B, 4>::from_data(
        TensorData::new(
            data,
            [states.len(), INPUT_PLANES, BOARD_RANKS, BOARD_FILES],
        ),
        device,
    )
}

fn policy_tensor<B: burn::prelude::Backend>(
    policies: &[[f32; MAX_NUM_ACTIONS]],
    device: &B::Device,
) -> Tensor<B, 2> {
    let mut data = Vec::with_capacity(policies.len() * MAX_NUM_ACTIONS);
    for policy in policies {
        data.extend_from_slice(policy);
    }

    Tensor::<B, 2>::from_data(
        TensorData::new(data, [policies.len(), MAX_NUM_ACTIONS]),
        device,
    )
}

fn value_tensor<B: burn::prelude::Backend>(
    values: &[[f32; 3]],
    device: &B::Device,
) -> Tensor<B, 2> {
    let mut data = Vec::with_capacity(values.len() * 3);
    for value in values {
        data.extend_from_slice(value);
    }

    Tensor::<B, 2>::from_data(TensorData::new(data, [values.len(), 3]), device)
}

fn cross_entropy<B: burn::tensor::backend::AutodiffBackend>(
    logits: Tensor<B, 2>,
    targets: Tensor<B, 2>,
) -> Tensor<B, 1> {
    let [batch_size, _] = targets.dims();
    let log_probs = log_softmax(logits, 1);
    targets
        .mul(log_probs)
        .sum()
        .neg()
        .div_scalar(batch_size as f32)
}

fn scalar<B: burn::prelude::Backend>(tensor: &Tensor<B, 1>) -> f32 {
    tensor.clone().into_data().to_vec::<f32>().unwrap()[0]
}

use rand::prelude::{SeedableRng, SliceRandom, StdRng};
