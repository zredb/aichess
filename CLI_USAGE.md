# AIChess 命令行工具使用指南

## 概述

AIChess CLI 是一个用于中国象棋 AI 训练和对弈的命令行工具，基于 AlphaZero 算法实现。

## 安装

```bash
cargo build --bin aichess-cli --release
```

编译后的可执行文件位于 `target/release/aichess-cli.exe`

## 功能

### 1. 模型训练 (train)

使用 AlphaZero 算法训练中国象棋 AI 模型。

#### 基本用法

```bash
aichess-cli train [选项]
```

#### 参数说明

| 参数 | 短选项 | 默认值 | 说明 |
|------|--------|--------|------|
| --log-dir | -d | ./logs | 日志和模型保存目录 |
| --iterations | -i | 10 | 训练迭代次数 |
| --games-per-train | -g | 100 | 每次训练的自对弈游戏数 |
| --games-to-keep | - | 500 | 保留的历史游戏数 |
| --batch-size | -b | 64 | 批次大小 |
| --epochs | -e | 5 | 训练轮数 |
| --num-explores | - | 800 | MCTS探索次数 |
| --workers | -w | 0 | 工作线程数（目前强制为0） |
| --learning-rate | -r | 0.001 | 学习率 |
| --policy-weight | - | 1.0 | 策略损失权重 |
| --value-weight | - | 1.0 | 价值损失权重 |
| --seed | - | 42 | 随机种子 |
| --hidden-size | - | 256 | 神经网络隐藏层大小 |
| --num-blocks | - | 7 | 神经网络残差块数量 |

#### 示例

**快速测试训练：**
```bash
aichess-cli train -i 1 -g 2 -d ./test_logs
```

**标准训练：**
```bash
aichess-cli train \
  -d ./my_training \
  -i 50 \
  -g 200 \
  -b 128 \
  -e 10 \
  --num-explores 800 \
  -r 0.001 \
  --hidden-size 256 \
  --num-blocks 7
```

**高性能训练（需要强大GPU）：**
```bash
aichess-cli train \
  -d ./pro_training \
  -i 100 \
  -g 500 \
  -b 256 \
  -e 20 \
  --num-explores 1600 \
  -r 0.0005 \
  --hidden-size 512 \
  --num-blocks 10
```

#### 输出说明

训练过程中会显示：
- 🚀 训练开始信息
- 📊 每轮迭代的统计信息：
  - 游戏数
  - 新鲜步数（新生成的数据）
  - 回放游戏数和步数（经验回放池）
  - 去重步数
  - 损失值（策略损失、价值损失、总损失）
- 💾 模型保存位置

训练完成后，模型保存在 `<log_dir>/models/` 目录下，文件名为 `model_0.ot`, `model_1.ot` 等。

### 2. 模型对弈 (play)

让两个训练好的模型相互对弈，评估模型性能。

#### 基本用法

```bash
aichess-cli play [选项]
```

#### 参数说明

| 参数 | 短选项 | 默认值 | 说明 |
|------|--------|--------|------|
| --model1 | -m1 | (必需) | 第一个模型路径 |
| --model2 | -m2 | (必需) | 第二个模型路径 |
| --games | -g | 10 | 对弈局数 |
| --num-explores | -n | 800 | MCTS探索次数 |
| --verbose | -v | false | 是否打印棋盘 |

#### 示例

```bash
aichess-cli play \
  -m1 ./logs/models/model_10.ot \
  -m2 ./logs/models/model_20.ot \
  -g 20 \
  -n 800 \
  -v
```

#### 输出说明

- 每局对弈的结果
- 最终统计：各模型胜率和和棋率

### 3. 人机对弈 (human)

人类玩家与 AI 模型进行对弈。

#### 基本用法

```bash
aichess-cli human [选项]
```

#### 参数说明

| 参数 | 短选项 | 默认值 | 说明 |
|------|--------|--------|------|
| --model | -m | (必需) | AI模型路径 |
| --color | -c | red | 玩家颜色 (red/black) |
| --num-explores | -n | 800 | MCTS探索次数 |
| --verbose | -v | false | 是否打印棋盘 |
| --save-pgn-file | - | None | 保存游戏到 PGN 文件 |

#### 示例

**执红先行：**
```bash
aichess-cli human -m ./logs/models/model_10.ot -c red -n 800
```

**执黑后行并保存 PGN：**
```bash
aichess-cli human -m ./logs/models/model_10.ot -c black -n 800 --save-pgn-file my_game.pgn
```

#### 操作说明

1. 游戏开始时会显示当前棋盘状态
2. 轮到玩家时，会列出所有合法走法及其编号
3. 输入走法编号选择要执行的走法
4. 输入 `quit` 或 `q` 退出游戏
5. AI 会自动思考并走棋
6. 游戏结束后，如果指定了 `--save-pgn-file`，游戏记录会自动保存

### 4. PGN 文件管理 (pgn)

查看和管理 PGN (Portable Game Notation) 格式的中国象棋棋谱文件。

#### 基本用法

```bash
aichess-cli pgn [选项]
```

#### 参数说明

| 参数 | 短选项 | 默认值 | 说明 |
|------|--------|--------|------|
| --file | -f | (必需) | PGN 文件路径 |
| --action | -a | show | 操作类型 (show/convert) |

#### 示例

**查看 PGN 文件：**
```bash
aichess-cli pgn -f games.pgn
```

**转换 PGN 格式（开发中）：**
```bash
aichess-cli pgn -f games.pgn -a convert
```

#### PGN 文件格式

PGN 文件包含游戏头部信息和走法记录：

```
[Event "比赛名称"]
[White "白方玩家"]
[Black "黑方玩家"]
[Result "1-0"]

1. 炮二平五 马8进7
2. 马二进三 车9平8
...
1-0
```

支持的头部标签：
- Event: 比赛名称
- White: 白方（红方）玩家
- Black: 黑方玩家
- Result: 游戏结果 (1-0, 0-1, 1/2-1/2, *)

## 训练建议

### 硬件要求

- **CPU**: 多核处理器（虽然目前使用单线程）
- **GPU**: 支持 WGPU 的显卡（NVIDIA/AMD/Intel）
- **内存**: 至少 8GB，推荐 16GB+
- **存储**: 根据训练规模，可能需要几十 GB

### 训练参数调优

#### 小规模测试
```bash
aichess-cli train -i 5 -g 50 --num-explores 400 -d ./test
```

#### 中等规模训练
```bash
aichess-cli train -i 50 -g 200 --num-explores 800 -b 128 -d ./medium
```

#### 大规模训练
```bash
aichess-cli train -i 200 -g 500 --num-explores 1600 -b 256 -e 20 -d ./large
```

### 超参数建议

1. **学习率 (learning-rate)**: 
   - 初始训练: 0.001
   - 微调: 0.0001-0.0005

2. **MCTS 探索次数 (num-explores)**:
   - 快速训练: 400
   - 标准训练: 800
   - 高质量训练: 1600+

3. **批次大小 (batch-size)**:
   - 小显存: 32-64
   - 中等显存: 128
   - 大显存: 256+

4. **网络结构**:
   - 小型: hidden-size=128, num-blocks=5
   - 中型: hidden-size=256, num-blocks=7
   - 大型: hidden-size=512, num-blocks=10+

## 常见问题

### Q: 训练很慢怎么办？

A: 
1. 减少 `--num-explores` 参数
2. 减少 `--games-per-train` 参数
3. 使用更小的网络结构（减小 hidden-size 和 num-blocks）

### Q: 如何继续之前的训练？

A: 使用相同的 log_dir，训练会自动从最新的 checkpoint 继续。

### Q: 模型文件在哪里？

A: 模型保存在 `<log_dir>/models/` 目录下。

### Q: 如何评估模型质量？

A: 使用 `play` 命令让不同迭代次数的模型对弈，观察胜率变化。

## 技术细节

- **算法**: AlphaZero (蒙特卡洛树搜索 + 深度神经网络)
- **后端**: Burn (Rust 深度学习框架)
- **GPU加速**: WGPU
- **文件格式**: Burn 的 MPK 格式

## 注意事项

⚠️ **当前限制**:
- 目前仅支持单线程训练（workers=0），以避免 GPU 设备的线程安全问题
- 人机对弈功能正在开发中，走法输入较为简化

## 开发者

如需扩展功能或修改代码，请参考：
- `src/bin/cli.rs` - 命令行接口实现
- `src/synthesis/` - AlphaZero 训练框架
- `src/cchess.rs` - 中国象棋游戏逻辑
