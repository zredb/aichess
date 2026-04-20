# AIChess GUI 技术选型建议

本文档详细分析了为 AIChess 项目添加 GUI 界面的技术方案。

---

## 🎯 核心需求分析

### 项目特点
- ✅ 已有完善的 CLI 接口
- ✅ 使用 burn + wgpu 进行深度学习
- ✅ 中国象棋对弈引擎
- ✅ AlphaZero 训练框架
- ✅ PGN 棋谱支持

### GUI 功能需求
1. **训练监控面板**
   - 实时显示训练进度
   - 动态损失曲线图
   - 超参数调整
   - 模型管理

2. **对弈界面**
   - 棋盘可视化
   - 走法输入（点击/拖拽）
   - AI 思考时间显示
   - 局面评估

3. **棋谱浏览器**
   - PGN 文件加载
   - 走法回放
   - 局面分析
   - 导出功能

4. **模型管理**
   - 模型列表
   - 版本对比
   - 导入/导出

---

## 🏆 推荐方案：egui + eframe

### 为什么选择 egui？

#### 1. **技术栈完美匹配** ⭐⭐⭐⭐⭐

```toml
# 你已有的依赖
burn = { version = "0.20.1", features = ["wgpu"] }

# egui 同样基于 wgpu，零冲突
egui = "0.31"
eframe = "0.31"
```

**优势**：
- ✅ 共享 wgpu 上下文
- ✅ 无需额外图形后端
- ✅ 内存效率高
- ✅ 渲染管线统一

#### 2. **学习曲线平缓** ⭐⭐⭐⭐⭐

```rust
// 极简示例 - 30 分钟即可上手
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AIChess");
            
            if ui.button("开始训练").clicked() {
                self.start_training();
            }
            
            ui.label(format!("迭代次数: {}", self.iteration));
        });
    }
}
```

#### 3. **即时模式优势** ⭐⭐⭐⭐

- ✅ 无状态管理复杂性
- ✅ UI 即代码，易于调试
- ✅ 热重载友好
- ✅ 响应式设计简单

#### 4. **丰富的组件库** ⭐⭐⭐⭐

```rust
// 内置常用组件
ui.button("按钮");
ui.checkbox(&mut self.enabled, "启用");
ui.slider(&mut self.value, 0.0..=100.0);
ui.text_edit_singleline(&mut self.text);
ui.add(egui::ProgressBar::new(self.progress));

// 绘图 API
ui.painter().circle_filled(pos, radius, color);
ui.painter().line_segment([start, end], stroke);
```

#### 5. **活跃的社区** ⭐⭐⭐⭐

- GitHub Stars: 15k+
- 每周更新
- 丰富的示例和文档
- Discord 社区活跃

---

## 📊 方案对比表

| 特性 | egui | iced | tauri | GTK4 |
|------|------|------|-------|------|
| **学习难度** | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **开发速度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **性能** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **灵活性** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **跨平台** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **wgpu 兼容** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **Rust 纯度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **包体积** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **社区规模** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

---

## 🚀 实施计划

### 阶段 1：基础框架（1-3 天）

**目标**：搭建 GUI 骨架，显示基本信息

```rust
// Cargo.toml 添加
[dependencies]
egui = "0.31"
eframe = "0.31"

[[bin]]
name = "aichess-gui"
path = "src/bin/gui.rs"
```

**功能**：
- ✅ 主窗口创建
- ✅ 菜单栏
- ✅ 基本布局
- ✅ 视图切换

**代码量**：~200 行

---

### 阶段 2：训练监控面板（3-5 天）

**目标**：可视化训练过程

**功能**：
- ✅ 训练参数配置界面
- ✅ 实时进度显示
- ✅ 损失曲线图（集成 plotters 或 egui_plot）
- ✅ 训练控制（开始/暂停/停止）
- ✅ 模型保存路径选择

**关键组件**：
```rust
use egui_plot::{Line, Plot, PlotPoints};

fn training_panel(&mut self, ui: &mut egui::Ui) {
    // 参数配置
    ui.group(|ui| {
        ui.heading("训练参数");
        ui.add(egui::Slider::new(&mut self.iterations, 1..=1000).text("迭代次数"));
        ui.add(egui::Slider::new(&mut self.games, 10..=1000).text("每轮游戏数"));
    });
    
    // 训练控制
    if ui.button("开始训练").clicked() {
        self.start_training();
    }
    
    // 损失曲线
    Plot::new("loss_plot")
        .view_aspect(2.0)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(self.loss_data.clone()));
        });
}
```

**代码量**：~400 行

---

### 阶段 3：对弈界面（5-7 天）

**目标**：完整的棋盘交互界面

**功能**：
- ✅ 棋盘绘制（9x10 网格）
- ✅ 棋子显示（SVG 或自定义绘制）
- ✅ 鼠标点击选子
- ✅ 合法走法高亮
- ✅ 走法执行动画
- ✅ AI 思考指示器
- ✅ 游戏状态显示

**关键技术**：
```rust
fn chess_board(&mut self, ui: &mut egui::Ui) {
    let response = ui.allocate_response(
        egui::vec2(450.0, 500.0),
        egui::Sense::click()
    );
    
    if let Some(pos) = response.hover_pos() {
        // 转换屏幕坐标到棋盘坐标
        let (file, rank) = screen_to_board(pos);
        
        if response.clicked() {
            self.handle_square_click(file, rank);
        }
    }
    
    // 绘制棋盘
    let painter = ui.painter_at(response.rect);
    self.draw_board(&painter, response.rect);
    self.draw_pieces(&painter, response.rect);
}
```

**代码量**：~600 行

---

### 阶段 4：棋谱浏览器（2-3 天）

**目标**：PGN 文件查看和分析

**功能**：
- ✅ PGN 文件选择
- ✅ 游戏列表显示
- ✅ 走法回放控制
- ✅ 当前局面显示
- ✅ 导出功能

**代码量**：~300 行

---

### 阶段 5：优化和完善（3-5 天）

**目标**：提升用户体验

**功能**：
- ✅ 主题切换（深色/浅色）
- ✅ 快捷键支持
- ✅ 设置持久化
- ✅ 错误处理优化
- ✅ 性能优化

**代码量**：~200 行

---

## 📦 完整依赖配置

```toml
[dependencies]
# GUI 核心
egui = "0.31"
eframe = "0.31"

# 图表绘制（可选，用于训练曲线）
egui_plot = "0.31"

# 或者使用 plotters（你已有）
plotters = "0.3.7"

# SVG 渲染（用于棋子图片）
resvg = "0.47.0"  # 你已有

# 异步任务处理
tokio = { version = "1.52.1", features = ["full"] }  # 你已有

# 其他已有依赖保持不变
burn = { version = "0.20.1", features = ["wgpu", "autodiff"] }
clap = { version = "4.4.7", features = ["derive"] }
# ...
```

---

## 🎨 UI 设计草图

### 主界面布局

```
┌─────────────────────────────────────────┐
│  菜单栏: 文件 编辑 视图 帮助             │
├──────────┬──────────────────────────────┤
│          │                              │
│  侧边栏   │      主内容区                 │
│          │                              │
│ • 训练   │   ┌────────────────────┐    │
│ • 对弈   │   │                    │    │
│ • 棋谱   │   │   棋盘/图表/列表    │    │
│ • 设置   │   │                    │    │
│          │   └────────────────────┘    │
│          │                              │
│          │   状态栏: 就绪/训练中/思考中  │
└──────────┴──────────────────────────────┘
```

### 训练面板

```
┌──────────────────────────────────────┐
│ 训练配置                              │
│ ┌──────────────────────────────────┐ │
│ │ 迭代次数: [====|====] 100        │ │
│ │ 每轮游戏: [====|====] 500        │ │
│ │ MCTS探索: [====|====] 800        │ │
│ │ 学习率:   0.001                  │ │
│ └──────────────────────────────────┘ │
│ [开始训练] [暂停] [停止]             │
├──────────────────────────────────────┤
│ 训练进度                              │
│ ████████████░░░░░░░░ 60%            │
│ 迭代: 60/100 | 游戏: 30000           │
├──────────────────────────────────────┤
│ 损失曲线                              │
│ ┌──────────────────────────────────┐ │
│ │     ╱╲                           │ │
│ │   ╱    ╲    🔴 策略损失          │ │
│ │ ╱        ╲  🔵 价值损失          │ │
│ │──────────  🟢 总损失             │ │
│ └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

### 对弈界面

```
┌──────────────────────────────────────┐
│ AI vs Human                           │
│ ┌──────────────────────────────────┐ │
│ │  车 马 象 士 将 士 象 马 车      │ │
│ │  ·  ·  ·  ·  ·  ·  ·  ·  ·      │ │
│ │  ·  炮  ·  ·  ·  ·  ·  炮  ·    │ │
│ │  卒  ·  卒  ·  卒  ·  卒  ·  卒  │ │
│ │  ·  ·  ·  ·  ·  ·  ·  ·  ·      │ │
│ │  ·  ·  ·  ·  ·  ·  ·  ·  ·      │ │
│ │  兵  ·  兵  ·  兵  ·  兵  ·  兵  │ │
│ │  ·  砲  ·  ·  ·  ·  ·  砲  ·    │ │
│ │  ·  ·  ·  ·  ·  ·  ·  ·  ·      │ │
│ │  俥 傌 相 仕 帥 仕 相 傌 俥      │ │
│ └──────────────────────────────────┘ │
│ 轮到: 红方 | AI思考中... ●●●         │
│ [悔棋] [提示] [新局]                 │
└──────────────────────────────────────┘
```

---

## 🔧 技术要点

### 1. 与现有代码集成

```rust
// 复用现有的 CLI 逻辑
use aichess::synthesis::AlphaZeroTrainer;
use aichess::cchess::CChess;

pub struct ChessApp {
    trainer: Option<AlphaZeroTrainer>,
    game: CChess,
    // ...
}

impl ChessApp {
    fn start_training(&mut self) {
        // 调用现有的训练逻辑
        tokio::spawn(async move {
            // 在后台线程运行训练
        });
    }
}
```

### 2. 异步任务处理

```rust
// 使用 tokio 处理长时间运行的任务
impl ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查训练状态
        if let Some(progress) = self.training_progress.try_recv() {
            self.update_ui(progress);
            ctx.request_repaint(); // 触发重绘
        }
    }
}
```

### 3. 棋盘绘制

```rust
fn draw_board(&self, painter: &egui::Painter, rect: egui::Rect) {
    let cell_width = rect.width() / 9.0;
    let cell_height = rect.height() / 10.0;
    
    // 绘制横线
    for i in 0..10 {
        let y = rect.min.y + i as f32 * cell_height;
        painter.line_segment(
            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        );
    }
    
    // 绘制竖线（楚河汉界断开）
    for i in 0..9 {
        let x = rect.min.x + i as f32 * cell_width;
        // 上半部分
        painter.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.min.y + 4.0 * cell_height)],
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        );
        // 下半部分
        painter.line_segment(
            [egui::pos2(x, rect.min.y + 5.0 * cell_height), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        );
    }
}
```

---

## ⚡ 性能考虑

### 1. 渲染优化
- ✅ egui 自动批处理绘制命令
- ✅ 只在内容变化时重绘
- ✅ 使用 `ctx.request_repaint()` 精确控制

### 2. 内存管理
- ✅ 及时清理旧的训练数据
- ✅ 使用环形缓冲区存储历史数据
- ✅ 避免在 UI 线程进行重型计算

### 3. 并发处理
```rust
// 训练在后台线程
let (tx, rx) = mpsc::channel();
std::thread::spawn(move || {
    // 运行训练
    for progress in training_iter {
        tx.send(progress).unwrap();
    }
});

// UI 线程接收更新
if let Ok(progress) = rx.try_recv() {
    self.progress = progress;
    ctx.request_repaint();
}
```

---

## 📚 学习资源

### 官方资源
- 📖 [egui 文档](https://docs.rs/egui/)
- 📖 [eframe 文档](https://docs.rs/eframe/)
- 🎥 [egui 视频教程](https://www.youtube.com/watch?v=NtUkr_z7l84)
- 💻 [示例代码](https://github.com/emilk/egui/tree/master/examples)

### 社区资源
- 💬 [egui Discord](https://discord.gg/vbuv9xan65)
- 📝 [Awesome egui](https://github.com/emilk/egui#examples)
- 🎨 [egui 主题生成器](https://egui.rs/)

### 相关项目
- ♟️ [Chess GUI with egui](https://github.com/search?q=egui+chess)
- 📊 [egui_plot 示例](https://github.com/emilk/egui_plot)

---

## 🎯 总结

### 为什么 egui 是最佳选择？

1. ✅ **技术栈一致** - wgpu 生态完美融合
2. ✅ **开发效率高** - 2-3 周完成 MVP
3. ✅ **学习成本低** - 1-2 天上手
4. ✅ **性能优秀** - 即时模式，高效渲染
5. ✅ **社区活跃** - 问题容易解决
6. ✅ **纯 Rust** - 无外部依赖困扰

### 预估投入

| 阶段 | 时间 | 代码量 | 难度 |
|------|------|--------|------|
| 基础框架 | 1-3 天 | 200 行 | ⭐⭐ |
| 训练面板 | 3-5 天 | 400 行 | ⭐⭐⭐ |
| 对弈界面 | 5-7 天 | 600 行 | ⭐⭐⭐⭐ |
| 棋谱浏览 | 2-3 天 | 300 行 | ⭐⭐⭐ |
| 优化完善 | 3-5 天 | 200 行 | ⭐⭐⭐ |
| **总计** | **14-23 天** | **~1700 行** | **⭐⭐⭐** |

### 下一步行动

1. **原型验证**（1 天）
   ```bash
   cargo add egui eframe
   # 创建简单的 hello world 窗口
   ```

2. **技术调研**（2-3 天）
   - 阅读 egui 文档
   - 运行示例代码
   - 测试棋盘绘制

3. **MVP 开发**（2 周）
   - 实现核心功能
   - 集成现有逻辑
   - 用户测试

4. **迭代优化**（持续）
   - 根据反馈改进
   - 添加高级功能
   - 性能调优

---

**结论**：对于 AIChess 项目，**egui + eframe 是最优选择**，能够在合理的时间内实现功能完善、性能优秀的 GUI 界面，同时保持与现有技术栈的高度一致性。🚀
