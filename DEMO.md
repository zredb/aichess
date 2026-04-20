# AIChess CLI 功能演示

本脚本展示了 AIChess 命令行工具的所有主要功能。

## 1. 查看帮助信息

```bash
# 主帮助
cargo run --bin aichess-cli -- --help

# 训练命令帮助
cargo run --bin aichess-cli -- train --help

# 人机对弈帮助
cargo run --bin aichess-cli -- human --help

# PGN 管理帮助
cargo run --bin aichess-cli -- pgn --help
```

## 2. 模型训练

### 快速测试（1次迭代，2局游戏）
```bash
cargo run --bin aichess-cli -- train \
  -i 1 \
  -g 2 \
  -d ./demo_logs
```

### 标准训练
```bash
cargo run --bin aichess-cli -- train \
  -d ./training_logs \
  -i 20 \
  -g 100 \
  -b 64 \
  -e 5 \
  --num-explores 800 \
  -r 0.001
```

## 3. 人机对弈

### 基本对弈
```bash
cargo run --bin aichess-cli -- human \
  -m ./demo_logs/models/model_1.ot \
  -c red
```

### 对弈并保存棋谱
```bash
cargo run --bin aichess-cli -- human \
  -m ./demo_logs/models/model_1.ot \
  -c black \
  --save-pgn-file my_game.pgn
```

**操作流程**:
1. 程序显示当前棋盘
2. 列出所有合法走法（带编号）
3. 输入走法编号（如 "0", "1", "2"）
4. 或输入 "q" 退出
5. 游戏结束后自动保存 PGN（如果指定）

## 4. PGN 文件管理

### 查看示例棋谱
```bash
cargo run --bin aichess-cli -- pgn -f test_games.pgn
```

输出示例:
```
📄 加载 PGN 文件: "test_games.pgn"
✅ 找到 2 个游戏

=== 游戏 1 ===
Event: 示例游戏
White: Player1
Black: Player2
Result: 1-0

走法:
1. 炮二平五  马8进7
2. 马二进三  车9平8
...
```

### 查看训练过程中保存的棋谱
```bash
# 假设你之前保存了一些游戏
cargo run --bin aichess-cli -- pgn -f my_games.pgn
```

## 5. 模型对弈

### 比较不同迭代的模型
```bash
cargo run --bin aichess-cli -- play \
  -m1 ./demo_logs/models/model_5.ot \
  -m2 ./demo_logs/models/model_10.ot \
  -g 10 \
  -n 800
```

输出:
```
🎮 开始模型对弈...
--- 第 1 局 ---
🏆 红方(模型1)获胜!
--- 第 2 局 ---
🤝 和棋!
...

📊 对弈结果统计:
   模型1 胜: 6 (60.0%)
   模型2 胜: 3 (30.0%)
   和棋: 1 (10.0%)
```

## 6. 完整工作流程示例

### 场景：从零开始训练并对弈

```bash
# 步骤 1: 训练一个基础模型
echo "=== 步骤 1: 训练模型 ==="
cargo run --bin aichess-cli -- train \
  -d ./my_chess_ai \
  -i 5 \
  -g 50 \
  --num-explores 400

# 步骤 2: 与训练的模型对弈并保存棋谱
echo "=== 步骤 2: 人机对弈 ==="
cargo run --bin aichess-cli -- human \
  -m ./my_chess_ai/models/model_5.ot \
  -c red \
  --save-pgn-file games/session1.pgn

# 步骤 3: 继续训练更多迭代
echo "=== 步骤 3: 继续训练 ==="
cargo run --bin aichess-cli -- train \
  -d ./my_chess_ai \
  -i 10 \
  -g 100 \
  --num-explores 800

# 步骤 4: 比较新旧模型
echo "=== 步骤 4: 模型对比 ==="
cargo run --bin aichess-cli -- play \
  -m1 ./my_chess_ai/models/model_5.ot \
  -m2 ./my_chess_ai/models/model_10.ot \
  -g 20

# 步骤 5: 查看保存的棋谱
echo "=== 步骤 5: 查看棋谱 ==="
cargo run --bin aichess-cli -- pgn -f games/session1.pgn
```

## 7. 高级用法

### 批量训练不同配置
```bash
# 小网络快速训练
cargo run --bin aichess-cli -- train \
  -d ./exp_small \
  -i 10 -g 50 \
  --hidden-size 128 --num-blocks 5

# 中等网络
cargo run --bin aichess-cli -- train \
  -d ./exp_medium \
  -i 20 -g 100 \
  --hidden-size 256 --num-blocks 7

# 大网络精细训练
cargo run --bin aichess-cli -- train \
  -d ./exp_large \
  -i 50 -g 200 \
  --hidden-size 512 --num-blocks 10
```

### 收集人类对弈数据
```bash
# 多次对弈，累积棋谱
for i in {1..10}; do
  echo "=== 第 $i 局 ==="
  cargo run --bin aichess-cli -- human \
    -m ./best_model.ot \
    -c red \
    --save-pgn-file human_vs_ai_collection.pgn
done
```

### 分析棋谱集合
```bash
# 查看所有收集的对局
cargo run --bin aichess-cli -- pgn -f human_vs_ai_collection.pgn

# 可以手动分析或编写脚本统计
# - 常见开局
# - 胜率统计
# - 平均步数等
```

## 8. 提示和技巧

### 性能优化
- 减少 `--num-explores` 可以加快训练速度
- 使用较小的批次大小适合显存有限的GPU
- 单线程模式下，CPU性能影响较小

### 调试技巧
- 使用 `-v` (verbose) 标志查看详细棋盘
- 检查 `logs/` 目录下的训练日志
- 模型文件在 `logs/models/` 目录

### 棋谱管理
- 定期备份重要的 PGN 文件
- 使用有意义的文件名（日期、对手等）
- 可以在 PGN 头部添加自定义标签

## 9. 常见问题

**Q: 训练很慢怎么办？**
A: 减少迭代次数、游戏数或 MCTS 探索次数

**Q: 如何继续之前的训练？**
A: 使用相同的 log_dir，程序会自动从最新 checkpoint 继续

**Q: PGN 文件在哪里？**
A: 在你使用 `--save-pgn-file` 参数时指定的位置

**Q: 如何查看某个特定模型的表现？**
A: 使用 `play` 命令让它与其他模型对弈

## 10. 下一步

- 尝试不同的训练配置
- 收集和分析大量对局数据
- 参与在线象棋社区分享棋谱
- 为项目贡献代码或提出建议

祝你在 AI 中国象棋的探索中取得成功！♟️
