use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::{BatchNormConfig, Dropout, Linear, LinearConfig, Relu};
use burn::nn::pool::AdaptiveAvgPool2d;
use burn::prelude::{Backend, Config, Module};
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
            conv1_bn: BatchNormConfig::new(self.num_classes),
            conv1_act: Relu::new(),

            conv2: Conv2dConfig::new([self.num_classes, self.num_classes], [3, 3]).init(device),
            conv2_bn: BatchNormConfig::new(self.num_classes),
            conv2_act: Relu::new(),

        }
    }
}

#[derive(Module, Debug)]
pub struct ResBlock<B: Backend> {
    conv1: Conv2d<B>,
    conv1_bn: B::BatchNorm,
    conv1_act: Relu,
    conv2: Conv2d<B>,
    conv2_bn: B::BatchNorm,
    conv2_act: Relu,
}

impl<B: Backend> ResBlock<B> {
    fn forward(&self, x: B::Tensor<4>) -> Self::ForwardOutput {
        let y = self.conv1.forward(x);
        let y = self.conv1_bn.forward(&x);
        let y = self.conv1_act.forward(&x);
        let y = self.conv2.forward(x);
        let y = self.conv2_bn.forward(&x);
        let y = self.conv2_act.forward(&x);
        let y = y + x;
        self.conv2_act.forward(&y)
    }
}


struct Net<B: Backend> {
    conv_block: Conv2d<B>,
    conv_block_bn: B::BatchNorm,
    conv_block_act: Relu,

    res_blocks: Vec<ResBlock<B>>,

    policy_conv: Conv2d<B>,
    policy_conv_bn: B::BatchNorm,
    policy_act: Relu,
    policy_fc: Linear<B>,

    value_conv: Conv2d<B>,
    value_conv_bn: B::BatchNorm,
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
            conv_block_bn: BatchNormConfig::new(self.num_classes),
            conv_block_act: Relu::new(),

            res_blocks,
            policy_conv: Conv2dConfig::new([self.num_classes, 16], [3, 3]).init(device),
            policy_conv_bn: BatchNormConfig::new(16),
            policy_act: Relu::new(),
            policy_fc: LinearConfig::new([16 * 9 * 10], 2086),

            value_conv: Conv2dConfig::new([self.num_classes, 8], [3, 3]).init(device),
            value_conv_bn: BatchNormConfig::new(8),
            value_act1: Relu::new(),
            value_fc1: LinearConfig::new([8 * 9 * 10], 256),
            value_act2: Relu::new(),
            value_fc2: LinearConfig::new([1256], 1),
        }
    }
}
impl<B: Backend> Net<B> {
    fn forward(&self, x: B::Tensor<4>) -> Self::ForwardOutput {
        let x = self.conv_block.forward(x);
        let x = self.conv_block_bn.forward(&x);
        let mut x = self.conv_block_act.forward(&x);
        for res_block in self.res_blocks.iter() {
            x = res_block.forward(x);
        }

        let policy = self.policy_conv.forward(x.clone());
        let policy = self.policy_conv_bn.forward(&policy);
        let policy = self.policy_act.forward(&policy);
        let policy = policy.reshape([-1, 16 * 9 * 10]);
        let policy = self.policy_fc.forward(policy);
        let policy=log_softmax(policy, 1);
        
        let value = self.value_conv.forward(x);
        let value = self.value_conv_bn.forward(&value);
        let value = self.value_act1.forward(&value);
        let value = value.reshape([-1, 8 * 9 * 10]);
        let value = self.value_fc1.forward(value);
        let value = self.value_act1.forward(&value);
        let value = self.value_fc2.forward(value);
        let value = tanh(value, );
        (policy, value)
    }
}


