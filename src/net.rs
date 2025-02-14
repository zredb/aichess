use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::{BatchNorm, BatchNormConfig, Linear, LinearConfig, Relu};
use burn::prelude::{Backend, Config, Module, Tensor};
use burn::tensor::activation::{log_softmax, tanh};

#[derive(Config, Debug)]
pub struct ResBlockConfig {
    num_classes: usize,
}
impl ResBlockConfig {
    /// Returns the initialized model.
    pub fn init<B: Backend>(&self, device: &B::Device) -> ResBlock<B> {
        ResBlock {
            conv1: Conv2dConfig::new([self.num_classes, self.num_classes], [3, 3]).init(device),
            conv1_bn: BatchNormConfig::new(self.num_classes).init(device),
            conv1_act: Relu::new(),

            conv2: Conv2dConfig::new([self.num_classes, self.num_classes], [3, 3]).init(device),
            conv2_bn: BatchNormConfig::new(self.num_classes).init(device),
            conv2_act: Relu::new(),

        }
    }
}

#[derive(Module, Debug)]
pub struct ResBlock<B: Backend> {
    conv1: Conv2d<B>,
    conv1_bn: BatchNorm<B,2>,
    conv1_act: Relu,
    conv2: Conv2d<B>,
    conv2_bn: BatchNorm<B,2>,
    conv2_act: Relu,
}

impl<B: Backend> ResBlock<B> {
    fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4>{
        let y = self.conv1.forward(x.clone());
        let y = self.conv1_bn.forward(y);
        let y = self.conv1_act.forward(y);
        let y = self.conv2.forward(y);
        let y = self.conv2_bn.forward(y);
        let y = self.conv2_act.forward(y);
        let y = y + x;
        self.conv2_act.forward(y)
    }
}


struct Net<B: Backend> {
    conv_block: Conv2d<B>,
    conv_block_bn: BatchNorm<B,2>,
    conv_block_act: Relu,

    res_blocks: Vec<ResBlock<B>>,

    policy_conv: Conv2d<B>,
    policy_conv_bn: BatchNorm<B,2>,
    policy_act: Relu,
    policy_fc: Linear<B>,

    value_conv: Conv2d<B>,
    value_conv_bn: BatchNorm<B,2>,
    value_act1: Relu,
    value_fc1: Linear<B>,
    value_act2: Relu,
    value_fc2: Linear<B>,

}

struct NetConfig {
    num_classes: usize,
    num_res_blocks: usize,
}
impl NetConfig {
    /// Returns the initialized model.
    pub fn init<B: Backend>(&self, device: &B::Device) -> Net<B> {
        let mut res_blocks = Vec::new();
        for i in 0..self.num_res_blocks {
            res_blocks.push(ResBlockConfig::new(self.num_classes).init(device));
        }
        Net {
            conv_block: Conv2dConfig::new([14, self.num_classes], [3, 3]).init(device),
            conv_block_bn: BatchNormConfig::new(self.num_classes).init(device),
            conv_block_act: Relu::new(),

            res_blocks,
            policy_conv: Conv2dConfig::new([self.num_classes, 16], [3, 3]).init(device),
            policy_conv_bn: BatchNormConfig::new(16).init(device),
            policy_act: Relu::new(),
            policy_fc: LinearConfig::new(16 * 9 * 10, 2086).init(device),

            value_conv: Conv2dConfig::new([self.num_classes, 8], [3, 3]).init(device),
            value_conv_bn: BatchNormConfig::new(8).init(device),
            value_act1: Relu::new(),
            value_fc1: LinearConfig::new(8 * 9 * 10, 256).init(device),
            value_act2: Relu::new(),
            value_fc2: LinearConfig::new(256, 1).init(device),
        }
    }
}
impl<B: Backend> Net<B> {
    fn forward(&self, x: Tensor<B,4>) ->(Tensor<B, 2>, Tensor<B, 2>) {
        let x = self.conv_block.forward(x);
        let x = self.conv_block_bn.forward(x);
        let mut x = self.conv_block_act.forward(x);
        for res_block in self.res_blocks.iter() {
            x = res_block.forward(x);
        }

        let policy = self.policy_conv.forward(x.clone());
        let policy = self.policy_conv_bn.forward(policy);
        let policy = self.policy_act.forward(policy);
        let policy = policy.reshape([-1, 16 * 9 * 10]);
        let policy = self.policy_fc.forward(policy);
        let policy = log_softmax(policy, 1);

        let value = self.value_conv.forward(x);
        let value = self.value_conv_bn.forward(value);
        let value = self.value_act1.forward(value);
        let value = value.reshape([-1, 8 * 9 * 10]);
        let value = self.value_fc1.forward(value);
        let value = self.value_act1.forward(value);
        let value = self.value_fc2.forward(value);
        let value = tanh(value);
        (policy, value)
    }
}


