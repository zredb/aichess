# AIChess GUI 使用指南

恭喜！AIChess 的图形界面已经成功构建。本文档将指导你如何使用 GUI 应用。

---

## 🚀 快速开始

### 运行 GUI 应用

```bash
cargo run --bin aichess-gui
```

或者在 release 模式下运行（性能更好）：

```bash
cargo run --release --bin aichess-gui
```

---

## 📋 界面概览

GUI 应用包含以下主要部分：

### 1. 菜单栏（顶部）
- **文件**: 退出应用
- **视图**: 切换不同功能面板
- **帮助**: 显示关于信息

### 2. 侧边栏（左侧）
快速导航到各个功能模块：
- 🎯 **训练** - AlphaZero 训练面板
- ⚔️ **对弈** - 模型对弈
- 👤 **人机** - 人机对弈
- 📜 **棋谱** - PGN 棋谱浏览

### 3. 主内容区（中央）
显示当前选中视图的内容

### 4. 状态栏（底部）
显示当前状态和版本信息

---

## 🎯 训练面板

### 功能说明

训练面板用于配置和监控 AlphaZero 模型的训练过程。

### 参数配置

| 参数 | 说明 | 推荐值 |
|------|------|--------|
| **日志目录** | 模型和日志保存路径 | `./logs` |
| **迭代次数** | 总训练轮数 | 50-200 |
| **每轮游戏数** | 每轮自对弈局数 | 100-500 |
| **MCTS 探索** | 每次决策的搜索次数 | 400-800 |
| **隐藏层大小** | 神经网络隐藏层维度 | 256-512 |
| **网络块数** | ResNet 块数量 | 7-10 |

### 训练控制

- **▶️ 开始训练**: 启动训练任务
- **⏸️ 暂停**: 暂停当前训练
- **⏹️ 停止**: 停止训练

### 训练曲线

实时显示训练损失曲线：
- 🔴 策略损失 (Policy Loss)
- 🔵 价值损失 (Value Loss)
- 🟢 总损失 (Total Loss)

### 使用示例

```
1. 设置日志目录: ./my_training
2. 调整参数:
   - 迭代次数: 50
   - 每轮游戏: 100
   - MCTS 探索: 400
3. 点击 "开始训练"
4. 观察训练曲线和进度
```

---

## ⚔️ 模型对弈

### 功能说明

让两个不同的 AI 模型进行对弈，评估模型强度。

### 参数配置

| 参数 | 说明 |
|------|------|
| **模型 1** | 第一个模型的路径 |
| **模型 2** | 第二个模型的路径 |
| **对弈局数** | 总共进行的局数 |
| **MCTS 探索** | AI 思考强度 |

### 统计信息

实时显示：
- 当前局数
- 模型 1 胜场
- 模型 2 胜场
- 平局数

### 使用场景

1. **模型对比**: 比较不同训练阶段的模型
2. **参数调优**: 测试不同 MCTS 探索次数的效果
3. **基准测试**: 建立模型强度基准

---

## 👤 人机对弈

### 功能说明

与 AI 模型进行对弈，测试模型实力。

### 参数配置

| 参数 | 说明 | 选项 |
|------|------|------|
| **模型路径** | AI 模型文件路径 | `.ot` 文件 |
| **玩家颜色** | 玩家选择的颜色 | 红方/黑方 |
| **AI 强度** | MCTS 探索次数 | 100-3200 |

### 游戏控制

- **🆕 新游戏**: 开始新的一局
- **↩️ 悔棋**: 撤销上一步（待实现）
- **💡 提示**: AI 建议走法（待实现）

### AI 强度说明

| MCTS 探索 | 强度等级 | 思考时间 |
|-----------|---------|---------|
| 100-200 | 初级 | 快 |
| 400-600 | 中级 | 中等 |
| 800-1200 | 高级 | 较慢 |
| 1600+ | 专家 | 慢 |

### 使用技巧

1. **初学者**: 从低强度开始（MCTS=200）
2. **练习**: 逐步提高强度
3. **挑战**: 尝试最高强度（MCTS=3200）

---

## 📜 棋谱浏览

### 功能说明

加载和查看 PGN 格式的棋谱文件。

### 功能特性

- 📂 加载 PGN 文件
- 📊 显示游戏统计
- 📝 查看走法记录
- ♻️ 走法回放（待实现）

### PGN 格式示例

```pgn
[Event "示例游戏"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. 炮二平五 马8进7
2. 马二进三 车9平8
...
1-0
```

### 使用方法

1. 点击 "📂 选择 PGN 文件"
2. 选择你的 PGN 文件
3. 查看棋谱内容

---

## 🎨 自定义和扩展

### 添加新功能

GUI 采用模块化设计，易于扩展：

```rust
// 1. 在 views.rs 中添加新视图
pub struct MyNewView {
    // 状态
}

impl MyNewView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // UI 代码
    }
}

// 2. 在 View 枚举中添加变体
pub enum View {
    // ...
    MyNew,
}

// 3. 在 app.rs 中集成
match self.current_view {
    // ...
    View::MyNew => {
        self.my_new_view.show(ui);
    }
}
```

### 自定义主题

egui 支持自定义主题和样式：

```rust
// 在 app.rs 的 new() 方法中
cc.egui_ctx.set_visuals(egui::Visuals::dark()); // 深色主题
// 或
cc.egui_ctx.set_visuals(egui::Visuals::light()); // 浅色主题
```

---

## ⚙️ 技术细节

### 架构设计

```
src/gui/
├── mod.rs          # 模块导出
├── app.rs          # 主应用逻辑
├── views.rs        # 各个视图实现
└── widgets.rs      # 自定义组件（预留）
```

### 依赖项

```toml
egui = "0.31"        # GUI 核心
eframe = "0.31"      # 应用框架
egui_plot = "0.31"   # 图表绘制
```

### 渲染后端

- 默认使用 **wgpu**（与 burn 共享）
- 自动检测系统最佳后端
- 支持 OpenGL/Vulkan/Metal/DirectX

---

## 🐛 常见问题

### Q1: GUI 窗口无法打开？

**A**: 检查以下几点：
1. 确保已正确编译：`cargo build --bin aichess-gui`
2. 检查显卡驱动是否最新
3. 尝试更新 wgpu：`cargo update -p wgpu`

### Q2: 训练曲线不显示？

**A**: 
- 确认已开始训练
- 检查是否有训练数据产生
- 刷新视图（切换标签页）

### Q3: 中文显示乱码？

**A**: 
- egui 默认支持 Unicode
- 如仍有问题，检查系统字体设置
- 可自定义字体（参考 egui 文档）

### Q4: 性能卡顿？

**A**:
1. 降低刷新率（修改 `request_repaint_after`）
2. 减少图表数据点
3. 使用 release 模式运行

---

## 📚 进阶开发

### 实现图形化棋盘

```rust
// 在 widgets.rs 中添加
pub fn chess_board(ui: &mut egui::Ui, board: &Board) {
    let response = ui.allocate_response(
        egui::vec2(450.0, 500.0),
        egui::Sense::click()
    );
    
    let painter = ui.painter_at(response.rect);
    
    // 绘制棋盘网格
    draw_grid(&painter, response.rect);
    
    // 绘制棋子
    draw_pieces(&painter, response.rect, board);
    
    // 处理点击事件
    if let Some(pos) = response.hover_pos() {
        if response.clicked() {
            handle_click(pos);
        }
    }
}
```

### 集成真实训练

```rust
// 在 TrainingView 中
fn start_training(&mut self) {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new().unwrap();
    let config = self.create_config();
    
    std::thread::spawn(move || {
        rt.block_on(async {
            // 调用实际的训练函数
            // train_model(config).await
        });
    });
}
```

### 添加快捷键

```rust
// 在 app.rs 的 update 方法中
if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
    // F5 刷新
    self.refresh();
}

if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
    // Ctrl+S 保存
    self.save();
}
```

---

## 🔗 相关资源

### 官方文档
- [egui 文档](https://docs.rs/egui/)
- [eframe 文档](https://docs.rs/eframe/)
- [egui_plot 文档](https://docs.rs/egui_plot/)

### 示例代码
- [egui 官方示例](https://github.com/emilk/egui/tree/master/examples)
- [egui_plot 示例](https://github.com/emilk/egui_plot)

### 社区
- [egui Discord](https://discord.gg/vbuv9xan65)
- [Rust GUI 讨论](https://users.rust-lang.org/c/gui/)

---

## 🎯 下一步计划

### 短期（1-2周）
- [ ] 实现图形化棋盘
- [ ] 集成真实训练逻辑
- [ ] 添加走法输入功能
- [ ] 完善 PGN 导入

### 中期（1-2月）
- [ ] 局面分析功能
- [ ] 开局库管理
- [ ] 多语言支持
- [ ] 主题定制

### 长期（3-6月）
- [ ] 在线对弈
- [ ] 棋谱分享
- [ ] AI 对战平台
- [ ] 移动端适配

---

## ✨ 总结

AIChess GUI 提供了一个直观、易用的界面来：
- ✅ 监控和管理模型训练
- ✅ 进行 AI 对弈和人机对弈
- ✅ 浏览和分析棋谱
- ✅ 可视化的训练曲线

享受你的中国象棋 AI 之旅！♟️🎮
