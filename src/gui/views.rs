use eframe::egui;
use std::path::PathBuf;

use crate::fen::Fen;
use crate::gui::widgets::ChessBoardWidget;

/// 视图类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Training,
    Play,
    Human,
    Pgn,
}

/// 训练视图
pub struct TrainingView {
    pub log_dir: String,
    pub iterations: usize,
    pub games_per_train: usize,
    pub num_explores: usize,
    pub hidden_size: usize,
    pub num_blocks: usize,
    pub is_training: bool,
    pub progress: f32,
    pub status: String,
    // 训练数据（用于绘图）
    pub loss_data: Vec<(f64, f64)>,
}

impl TrainingView {
    pub fn new() -> Self {
        Self {
            log_dir: "./logs".to_string(),
            iterations: 50,
            games_per_train: 100,
            num_explores: 400,
            hidden_size: 256,
            num_blocks: 7,
            is_training: false,
            progress: 0.0,
            status: "就绪".to_string(),
            loss_data: vec![],
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎯 AlphaZero 训练");
        ui.add_space(10.0);
        
        // 训练参数配置
        egui::Grid::new("training_params")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("日志目录:");
                ui.text_edit_singleline(&mut self.log_dir);
                ui.end_row();
                
                ui.label("迭代次数:");
                ui.add(egui::Slider::new(&mut self.iterations, 1..=1000));
                ui.end_row();
                
                ui.label("每轮游戏数:");
                ui.add(egui::Slider::new(&mut self.games_per_train, 10..=1000));
                ui.end_row();
                
                ui.label("MCTS 探索次数:");
                ui.add(egui::Slider::new(&mut self.num_explores, 100..=3200));
                ui.end_row();
                
                ui.label("隐藏层大小:");
                ui.add(egui::Slider::new(&mut self.hidden_size, 64..=1024).logarithmic(true));
                ui.end_row();
                
                ui.label("网络块数:");
                ui.add(egui::Slider::new(&mut self.num_blocks, 1..=20));
                ui.end_row();
            });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 训练控制
        ui.horizontal(|ui| {
            if !self.is_training {
                if ui.button("▶️ 开始训练").clicked() {
                    self.start_training();
                }
            } else {
                if ui.button("⏸️ 暂停").clicked() {
                    self.status = "已暂停".to_string();
                }
                if ui.button("⏹️ 停止").clicked() {
                    self.stop_training();
                }
            }
            
            ui.label(format!("进度: {:.1}%", self.progress * 100.0));
        });
        
        ui.add_space(10.0);
        
        // 进度条
        ui.add(egui::ProgressBar::new(self.progress)
            .text(if self.is_training { 
                format!("{:.0}%", self.progress * 100.0) 
            } else { 
                "未开始".to_string() 
            }));
        
        ui.add_space(10.0);
        ui.label(format!("状态: {}", self.status));
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 训练曲线图
        ui.heading("📊 训练曲线");
        ui.add_space(10.0);
        
        if self.loss_data.is_empty() {
            ui.label("暂无训练数据");
        } else {
            use egui_plot::{Line, Plot, PlotPoints};
            
            let points: PlotPoints = self.loss_data.iter()
                .map(|&(x, y)| [x, y])
                .collect::<Vec<_>>()
                .into();
            let line = Line::new(points).name("总损失");
            
            Plot::new("loss_plot")
                .view_aspect(2.0)
                .label_formatter(|name, value| {
                    format!("{}: {:.4}", name, value.y)
                })
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        }
    }
    
    fn start_training(&mut self) {
        self.is_training = true;
        self.progress = 0.0;
        self.status = "训练中...".to_string();
        self.loss_data.clear();
        
        // TODO: 在实际实现中，这里会启动后台训练任务
        // 目前只是模拟进度
    }
    
    fn stop_training(&mut self) {
        self.is_training = false;
        self.status = "已停止".to_string();
    }
}

/// 模型对弈视图
pub struct PlayView {
    pub model1_path: String,
    pub model2_path: String,
    pub num_games: usize,
    pub num_explores: usize,
    pub is_playing: bool,
    pub current_game: usize,
    pub wins_p1: usize,
    pub wins_p2: usize,
    pub draws: usize,
}

impl PlayView {
    pub fn new() -> Self {
        Self {
            model1_path: "./models/model1.ot".to_string(),
            model2_path: "./models/model2.ot".to_string(),
            num_games: 10,
            num_explores: 400,
            is_playing: false,
            current_game: 0,
            wins_p1: 0,
            wins_p2: 0,
            draws: 0,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚔️ 模型对弈");
        ui.add_space(10.0);
        
        // 模型配置
        egui::Grid::new("play_params")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("模型 1:");
                ui.text_edit_singleline(&mut self.model1_path);
                ui.end_row();
                
                ui.label("模型 2:");
                ui.text_edit_singleline(&mut self.model2_path);
                ui.end_row();
                
                ui.label("对弈局数:");
                ui.add(egui::Slider::new(&mut self.num_games, 1..=100));
                ui.end_row();
                
                ui.label("MCTS 探索:");
                ui.add(egui::Slider::new(&mut self.num_explores, 100..=3200));
                ui.end_row();
            });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 控制按钮
        if !self.is_playing {
            if ui.button("▶️ 开始对弈").clicked() {
                self.start_play();
            }
        } else {
            if ui.button("⏹️ 停止").clicked() {
                self.stop_play();
            }
        }
        
        ui.add_space(10.0);
        
        // 统计信息
        ui.group(|ui| {
            ui.heading("📊 对弈统计");
            ui.add_space(5.0);
            ui.label(format!("当前局数: {}/{}", self.current_game, self.num_games));
            ui.label(format!("模型 1 胜: {}", self.wins_p1));
            ui.label(format!("模型 2 胜: {}", self.wins_p2));
            ui.label(format!("平局: {}", self.draws));
        });
    }
    
    fn start_play(&mut self) {
        self.is_playing = true;
        self.current_game = 0;
        self.wins_p1 = 0;
        self.wins_p2 = 0;
        self.draws = 0;
        // TODO: 启动后台对弈任务
    }
    
    fn stop_play(&mut self) {
        self.is_playing = false;
    }
}

/// 人机对弈视图
pub struct HumanView {
    pub model_path: String,
    pub player_color: String,
    pub num_explores: usize,
    pub game_status: String,
    pub chess_board: ChessBoardWidget,
    pub current_fen: Fen,
}

impl HumanView {
    pub fn new() -> Self {
        Self {
            model_path: "./models/model.ot".to_string(),
            player_color: "红方".to_string(),
            num_explores: 800,
            game_status: "未开始".to_string(),
            // chess_board: ChessBoardWidget::new(),  // 暂时注释掉
            chess_board: ChessBoardWidget::new().with_size(450.0, 500.0),
            // current_fen: Fen::init(),  // 暂时注释掉
            current_fen: Fen::new("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"),
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("👤 人机对弈");
        ui.add_space(10.0);
        
        // 配置 - 使用紧凑的 Grid
        egui::Grid::new("human_params")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("模型路径:");
                ui.text_edit_singleline(&mut self.model_path);
                ui.end_row();
                
                ui.label("玩家颜色:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.player_color, "红方".to_string(), "红方（先手）");
                    ui.radio_value(&mut self.player_color, "黑方".to_string(), "黑方（后手）");
                });
                ui.end_row();
                
                ui.label("AI 强度:");
                let explores = self.num_explores;
                ui.add(egui::Slider::new(&mut self.num_explores, 100..=3200)
                    .text(format!("{}", explores)));
                ui.end_row();
            });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 棋盘区域 - 使用 SVG 图形化棋盘
        ui.heading("棋盘");
        ui.add_space(5.0);
        
        let ctx = ui.ctx().clone();
        
        // 水平布局 - 棋盘和走法列表
        ui.horizontal(|ui| {
            // 左侧棋盘 - 使用 allocate_ui_with_layout 分配空间
            let ctx = ui.ctx().clone();
            
            // 计算期望的棋盘尺寸(基于窗口宽度)
            let window_width = ui.available_width();
            let desired_width = (window_width * 0.6).max(450.0);
            let aspect_ratio = 502.0 / 452.0;
            let desired_height = desired_width * aspect_ratio;
            
            // 分配确切的空间
            ui.allocate_ui_with_layout(
                egui::vec2(desired_width, desired_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    if let Some(mv) = self.chess_board.show(ui, &ctx, &self.current_fen) {
                        self.game_status = format!("走法: {}", mv);
                    }
                }
            );
            
            ui.add_space(10.0);
            
            // 右侧走法列表
            ui.vertical(|ui| {
                ui.label("走法记录");
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.small("(走法将在这里显示)");
                    });
            });
        });
        
        ui.add_space(10.0);
        
        // 控制按钮
        ui.horizontal(|ui| {
            if ui.button("🆕 新游戏").clicked() {
                self.game_status = "游戏进行中".to_string();
            }
            if ui.button("↩️ 悔棋").clicked() {
                // TODO: 实现悔棋
            }
            if ui.button("💡 提示").clicked() {
                // TODO: 实现提示
            }
        });
        
        ui.add_space(10.0);
        ui.label(format!("状态: {}", self.game_status));
    }
}

/// 棋谱浏览视图
pub struct PgnView {
    pub pgn_file: Option<PathBuf>,
    pub pgn_content: String,
    pub loaded_games: usize,
}

impl PgnView {
    pub fn new() -> Self {
        Self {
            pgn_file: None,
            pgn_content: "请选择 PGN 文件".to_string(),
            loaded_games: 0,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("📜 棋谱浏览");
        ui.add_space(10.0);
        
        // 文件选择
        ui.horizontal(|ui| {
            if ui.button("📂 选择 PGN 文件").clicked() {
                // TODO: 实现文件选择对话框
                self.load_sample_pgn();
            }
            
            if let Some(path) = &self.pgn_file {
                ui.label(format!("当前文件: {:?}", path.file_name().unwrap_or_default()));
            }
        });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 统计信息
        ui.label(format!("已加载游戏数: {}", self.loaded_games));
        
        ui.add_space(10.0);
        
        // PGN 内容显示
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.code(&self.pgn_content);
            });
    }
    
    fn load_sample_pgn(&mut self) {
        // 示例 PGN 数据
        self.pgn_content = r#"[Event "示例游戏"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. 炮二平五 马8进7
2. 马二进三 车9平8
3. 车一平二 马2进3
4. 兵七进一 卒7进1
5. 车二进六 炮8平9
6. 车二平三 炮9退1
7. 马八进七 士4进5
8. 炮八平九 炮9平7
9. 车三平四 马7进8
10. 车九平八 车1平2
1-0
"#.to_string();
        self.loaded_games = 1;
        self.pgn_file = Some(PathBuf::from("sample.pgn"));
    }
}
